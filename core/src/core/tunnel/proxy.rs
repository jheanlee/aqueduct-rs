use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use nanoid::nanoid;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::select;
use crate::core::message::message::{Message, MessageType};
use crate::core::socket::io::send_message;
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::model::{Flags, ProxyClient, TunnelClient, TunnelStatus};

pub async fn tunnel_client_proxy_control(flags: Flags, tunnel_client: Arc<TunnelClient>, tunnel_status: Arc<TunnelStatus>) -> Result<(), TunnelError> {
  //  assign a port
  let mut tcp_listener = None;
  {
    let mut available_ports = tunnel_status.available_ports.write().await;

    while let Some(new_port) = available_ports.pop_front() {
      if let Ok(new_tcp_listener) = TcpListener::bind(format!("{}:{}", tunnel_status.host, new_port)).await {
        tcp_listener = Some(new_tcp_listener);
        break;
      }
    }
  }

  if let Some(tcp_listener) = tcp_listener {
    //  send port number to client
    let port = tcp_listener.local_addr()?.port();
    let message = Message::new(MessageType::Port, format!("{port}"));
    {
      let mut stream = tunnel_client.stream.lock().await;
      let res = send_message(stream.deref_mut(), &message).await;
      if let Err(error) = res {
        //  TODO log error
        flags.local_cancellation_token.cancel();
        Err(error)?;
      }
    }

    //  TODO log listening on new port (debug)

    loop {
      select! {
        result = tcp_listener.accept() => {
          match result {
            Ok((external_client_stream, external_client_addr)) => {
              //  generate id
              let id = nanoid!();
              let message = Message::new(MessageType::Proxy, id.clone());
              {
                //  insert into queue
                let mut proxy_queue = tunnel_status.proxy_queue.write().await;
                proxy_queue.insert(
                  id,
                  ProxyClient {
                    external_client_stream: external_client_stream,
                    external_client_addr: external_client_addr,
                    proxy_control_client_addr: tunnel_client.addr.clone(),
                    proxy_control_server_addr: SocketAddr::new(tunnel_status.host.parse().unwrap(), port),
                  }
                );
              }
              {
                //  notify client of the new user
                let mut stream = tunnel_client.stream.lock().await;
                let res = send_message(stream.deref_mut(), &message).await;
                if let Err(error) = res {
                  //  TODO log write error
                  flags.local_cancellation_token.cancel();
                  break;
                }
              }
            }
            Err(error) => {
              //  TODO log error
              flags.local_cancellation_token.cancel();
              break;
            }
          }
        }
        _client_cancealled = flags.local_cancellation_token.cancelled() => {
          break;
        },
      }
    }
    //  fd closed on drop
  } else {
    //  TODO log not available
    let message = Message::new(MessageType::Error, "no ports available".to_string());
    let mut stream = tunnel_client.stream.lock().await;
    let _res = send_message(stream.deref_mut(), &message).await;
    flags.local_cancellation_token.cancel();
  }
  Ok(())
}

pub async fn tunnel_client_proxy(flags: Flags, tunnel_client: Arc<TunnelClient>, mut proxy_client: ProxyClient, tunnel_status: Arc<TunnelStatus>) -> Result<(), TunnelError> {
  //  TODO log proxy started
  let mut tunnel_buffer = [0u8; 32768];
  let mut external_buffer = [0u8; 32768];

  //  only this thread would access this stream
  let mut tunnel_client_stream = tunnel_client.stream.lock().await;

  loop {
    tunnel_buffer.fill(0u8);
    external_buffer.fill(0u8);
    select! {
      tunnel_client_read = tunnel_client_stream.read(&mut tunnel_buffer) => {
        //  client (service) -> external_client
        match tunnel_client_read {
          Ok(bytes_read) => {
            let write_result = proxy_client.external_client_stream.write_all(&tunnel_buffer[..bytes_read]).await;
            match write_result {
              Ok(_) => {
                //  TODO usage counter
              }
              Err(error) => {
                //  TODO log closed (debug)
                break;
              }
            }
          }
          Err(error) => {
            //  TODO log closed (debug)
            break;
          }
        }
      }
      external_client_read = proxy_client.external_client_stream.read(&mut external_buffer) => {
        //  external_client -> client (service)
        match external_client_read {
          Ok(bytes_read) => {
            let write_result = tunnel_client_stream.write_all(&tunnel_buffer[..bytes_read]).await;
            match write_result {
              Ok(_) => {
                //  TODO usage counter
              }
              Err(error) => {
                //  TODO log closed (debug)
                break;
              }
            }
          }
          Err(error) => {
            //  TODO log closed (debug)
            break;
          }
        }
      }
      _client_cancealled = flags.local_cancellation_token.cancelled() => {
        break;
      }
    }
  }

  flags.local_cancellation_token.cancel();
  let _shutdown_status = proxy_client.external_client_stream.shutdown().await;
  //  TODO log proxy ended
  Ok(())
}