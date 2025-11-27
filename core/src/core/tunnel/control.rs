use std::ops::DerefMut;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::watch;
use crate::config::tunnel::TUNNEL_CLIENT_HEARTBEAT_TIMEOUT;
use crate::core::message::message::{Message, MessageType, ProxyMessage, ServiceMessage};
use crate::core::socket::io::{read_message, send_message};
use crate::core::tunnel::model::{ClientType, Flags, TunnelClient, TunnelStatus};
use crate::core::tunnel::proxy::{tunnel_client_proxy, tunnel_client_proxy_control};

pub async fn tunnel_client_control(flags: Flags, tunnel_client: Arc<TunnelClient>, tunnel_status: Arc<TunnelStatus>) {
  let mut client_type: Option<ClientType> = None;
  let mut buffer = [0u8; 1024];
  let (heartbeat_tx, heartbeat_rx) = watch::channel(false);

  let mut tunnel_client_heartbeat_thread = None;
  let mut tunnel_client_proxy_control_thread = None;
  let mut tunnel_client_proxy_thread = None;

  loop {
    let read_future = async {
      let mut guard = tunnel_client.stream.lock().await;
      read_message(guard.deref_mut(), buffer.as_mut()).await
    };

    select! {
      result = read_future => {
        match result {
          Ok(message) => {
            match message.message_type {
              MessageType::Heartbeat => {
                heartbeat_tx.send_replace(true);
              },
              MessageType::Service => {
                match serde_json::from_str::<ServiceMessage>(message.message_string.as_str()) {
                  Ok(client_info) => {
                    //  TODO authentication
                    client_type = Some(ClientType::Service);
                    tunnel_client_heartbeat_thread = Some(tokio::spawn(
                      tunnel_client_heartbeat(
                        flags.clone(),
                        tunnel_client.clone(),
                        (heartbeat_tx.clone(), heartbeat_rx.clone())
                      )
                    ));
                    tunnel_client_proxy_control_thread = Some(tokio::spawn(
                      tunnel_client_proxy_control(
                        flags.clone(),
                        tunnel_client.clone(),
                        tunnel_status.clone()
                      )
                    ));
                  }
                  Err(_) => {

                    }
                  }
                },
              MessageType::Proxy => {
                match serde_json::from_str::<ProxyMessage>(message.message_string.as_str()) {
                  Ok(client_info) => {
                    if let Some(proxy_client) = tunnel_status.proxy_queue.write().await.remove(&client_info.proxy_id) {
                      client_type = Some(ClientType::Proxy);
                      tunnel_client_proxy_thread = Some(tokio::spawn(
                        tunnel_client_proxy(
                          flags.clone(),
                          tunnel_client.clone(),
                          proxy_client,
                          tunnel_status.clone()
                        )
                      ));
                    } else {
                      //  TODO log denied access
                      let message = Message::new(MessageType::Error, String::from("access denied"));
                      let _res = send_message(tunnel_client.stream.lock().await.deref_mut(), &message).await;
                    }
                    break;
                  }
                  Err(_) => {}
                }
              },
              MessageType::Port => {

              },
              MessageType::Close => {
                flags.local_cancellation_token.cancel();
                break;
              }
              MessageType::Error => {
                flags.local_cancellation_token.cancel();
                break;
              }
            }
          }
          Err(error) => {
            flags.local_cancellation_token.cancel();
            break;
          }
        }
      },
      _global_cancalled = flags.global_cancellation_token.cancelled() => {
        flags.local_cancellation_token.cancel();
        break;
      },
      _client_cancealled = flags.local_cancellation_token.cancelled() => {
        break;
      },
    }
  }

  if let Some(thread) = tunnel_client_proxy_thread {
    let _ = thread.await;
  }

  if let Some(thread) = tunnel_client_heartbeat_thread {
    let _ = thread.await;
  }

  if let Some(thread) = tunnel_client_proxy_control_thread {
    let _ = thread.await;
  }

  let _shutdown_status = tunnel_client.stream.lock().await.shutdown().await;
  //  TODO log("Connection with client {client.addr} has ended")
}

pub async fn tunnel_client_heartbeat(flags: Flags, tunnel_client: Arc<TunnelClient>, (heartbeat_tx, mut heartbeat_rx): (watch::Sender<bool>, watch::Receiver<bool>)) {
  let message = Message::new(MessageType::Heartbeat, String::new());

  loop {
    //  wait for heartbeat
    let value = select! {
      heartbeat_changed = heartbeat_rx.changed() => {
        if !heartbeat_changed.is_err() {
          Some(*heartbeat_rx.borrow())
        } else {
          flags.local_cancellation_token.cancel();
          None
        }
      },
      _global_cancalled = flags.global_cancellation_token.cancelled() => None,
      _client_cancealled = flags.local_cancellation_token.cancelled() => None,
      _sleep = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => None
    };

    //  sleep until next cycle
    match value {
      Some(value) if value => {
        select! {
          _global_cancalled = flags.global_cancellation_token.cancelled() => { break; },
          _client_cancealled = flags.local_cancellation_token.cancelled() => { break; },
          _sleep = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => {},
        }
      }
      _ => {
        break;
      }
    }

    //  send heartbeat
    let write_future = async {
      let mut guard = tunnel_client.stream.lock().await;
      heartbeat_tx.send_replace(false);
      heartbeat_rx.borrow_and_update();
      send_message(guard.deref_mut(), &message).await
    };

    select! {
      write_result = write_future => {
        match write_result {
          Ok(_) => {},
          Err(_error) => {
            //  TODO log
            flags.local_cancellation_token.cancel();
            break;
          }
        }
      },
      _global_cancalled = flags.global_cancellation_token.cancelled() => { break; },
      _client_cancealled = flags.local_cancellation_token.cancelled() => { break; },
    }
  }
}