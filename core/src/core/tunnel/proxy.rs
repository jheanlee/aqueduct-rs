use crate::common::log::{Level, log};
use crate::core::message::message::{Message, MessageType};
use crate::core::socket::io::send_message;
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::error::TunnelError::NoPortsAvailable;
use crate::core::tunnel::model::{Flags, ProxyClient, TunnelClient, TunnelStatus};
use nanoid::nanoid;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::select;

pub async fn tunnel_client_proxy_control(
    flags: Flags,
    tunnel_client: Arc<TunnelClient>,
    tunnel_status: Arc<TunnelStatus>,
) -> Result<(), TunnelError> {
    //  assign a port
    let mut tcp_listener = None;
    {
        let mut available_ports = tunnel_status.available_ports.write().await;

        while let Some(new_port) = available_ports.pop_front() {
            if let Ok(new_tcp_listener) =
                TcpListener::bind(format!("{}:{}", tunnel_status.host, new_port)).await
            {
                tcp_listener = Some(new_tcp_listener);
                break;
            }
        }
    }

    let Some(tcp_listener) = tcp_listener else {
        log(
            Level::Warning,
            "No ports available",
            "core::tunnel::proxy::tunnel_client_proxy_control",
        )
        .await;
        let message = Message::new(MessageType::Error, "no ports available".to_string());
        let mut stream = tunnel_client.stream_tx.lock().await;
        let _res = send_message(stream.deref_mut(), &message).await;
        flags.local_cancellation_token.cancel();
        Err(NoPortsAvailable)?
    };

    //  send port number to client
    let port = tcp_listener.local_addr()?.port();
    let message = Message::new(MessageType::Port, format!("{port}"));
    {
        let mut stream = tunnel_client.stream_tx.lock().await;
        let res = send_message(stream.deref_mut(), &message).await;
        if let Err(error) = res {
            log(
                Level::Debug,
                format!(
                    "Unable to send assigned port number to {}: {:?}",
                    tunnel_client.addr.to_string(),
                    error
                )
                .as_str(),
                "core::tunnel::proxy::tunnel_client_proxy_control",
            )
            .await;
            flags.local_cancellation_token.cancel();
            Err(error)?;
        }
    }

    log(
        Level::Info,
        format!(
            "Tunnel port {} assigned to client {}, started listening",
            port,
            tunnel_client.addr.to_string()
        )
        .as_str(),
        "core::tunnel::proxy::tunnel_client_proxy_control",
    )
    .await;

    //  accept external connections
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
                            let (external_client_stream_rx, external_client_stream_tx) = external_client_stream.into_split();
                            let mut proxy_queue = tunnel_status.proxy_queue.write().await;
                            proxy_queue.insert(
                                id,
                                ProxyClient {
                                    external_client_stream_rx: external_client_stream_rx,
                                    external_client_stream_tx: external_client_stream_tx,
                                    external_client_addr: external_client_addr,
                                    proxy_control_client_addr: tunnel_client.addr.clone(),
                                    proxy_control_server_addr: SocketAddr::new(
                                        tunnel_status.host.parse().unwrap_or_else(|_| {unreachable!()}),
                                        port
                                    ),
                                }
                            );
                        }
                        {
                            //  notify client of the new user
                            let mut stream = tunnel_client.stream_tx.lock().await;
                            let res = send_message(stream.deref_mut(), &message).await;
                            if let Err(error) = res {
                                log(
                                    Level::Warning,
                                    format!(
                                        "Unable to send external connection request to client {}: {:?}",
                                        tunnel_client.addr.to_string(),
                                        error
                                    )
                                    .as_str(),
                                    "core::tunnel::proxy::tunnel_client_proxy_control"
                                )
                                .await;

                                flags.local_cancellation_token.cancel();
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        log(Level::Warning, format!("Unable to accept external connection for client {}: {:?}", tunnel_client.addr.to_string(), error).as_str(), "core::tunnel::proxy::tunnel_client_proxy_control").await;
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
    Ok(())
}

pub async fn tunnel_client_proxy(
    flags: Flags,
    tunnel_client: Arc<TunnelClient>,
    mut proxy_client: ProxyClient,
    tunnel_status: Arc<TunnelStatus>,
) -> Result<(), TunnelError> {
    log(
        Level::Debug,
        format!(
            "TCP proxying started {} <=> {}",
            tunnel_client.addr.to_string(),
            proxy_client.external_client_addr.to_string()
        )
        .as_str(),
        "core::tunnel::proxy::tunnel_client_proxy",
    )
    .await;

    let mut tunnel_buffer = [0u8; 32768];
    let mut external_buffer = [0u8; 32768];

    //  only this thread would access this stream
    let mut tunnel_client_stream_rx = tunnel_client.stream_rx.lock().await;
    let mut tunnel_client_stream_tx = tunnel_client.stream_tx.lock().await;

    loop {
        tunnel_buffer.fill(0u8);
        external_buffer.fill(0u8);
        select! {
            tunnel_client_read = tunnel_client_stream_rx.read(&mut tunnel_buffer) => {
                //  client (service) -> external_client
                match tunnel_client_read {
                    Ok(bytes_read) => {
                        println!("{}",String::from_utf8_lossy(&tunnel_buffer));
                        let write_result = proxy_client.external_client_stream_tx.write_all(&tunnel_buffer[..bytes_read]).await;
                        match write_result {
                            Ok(_) => {
                                //  TODO usage counter
                            }
                            Err(error) => {
                                log(
                                    Level::Debug,
                                    format!(
                                        "Proxy write failed {} => {}: {:?}",
                                        tunnel_client.addr.to_string(),
                                        proxy_client.external_client_addr.to_string(),
                                        error
                                    )
                                    .as_str(),
                                    "core::tunnel::proxy::tunnel_client_proxy"
                                )
                                .await;
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        log(
                            Level::Debug,
                            format!(
                                "Proxy read failed {} => {}: {:?}",
                                tunnel_client.addr.to_string(),
                                proxy_client.external_client_addr.to_string(),
                                error
                            )
                            .as_str(),
                            "core::tunnel::proxy::tunnel_client_proxy"
                        )
                        .await;
                        break;
                    }
                }
            }
            external_client_read = proxy_client.external_client_stream_rx.read(&mut external_buffer) => {
                //  external_client -> client (service)
                match external_client_read {
                    Ok(bytes_read) => {
                        let write_result = tunnel_client_stream_tx.write_all(&external_buffer[..bytes_read]).await;
                        match write_result {
                            Ok(_) => {
                                //  TODO usage counter
                            }
                            Err(error) => {
                                log(
                                    Level::Debug,
                                    format!(
                                        "Proxy write failed {} <= {}: {:?}",
                                        tunnel_client.addr.to_string(),
                                        proxy_client.external_client_addr.to_string(),
                                        error
                                    )
                                    .as_str(),
                                    "core::tunnel::proxy::tunnel_client_proxy"
                                )
                                .await;
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        log(
                            Level::Debug,
                            format!(
                                "Proxy read failed {} <= {}: {:?}",
                                tunnel_client.addr.to_string(),
                                proxy_client.external_client_addr.to_string(),
                                error
                            )
                            .as_str(),
                            "core::tunnel::proxy::tunnel_client_proxy"
                        )
                        .await;
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
    let _shutdown_status = proxy_client.external_client_stream_tx.shutdown().await;
    log(
        Level::Debug,
        format!(
            "TCP proxying ended {} <=> {}",
            tunnel_client.addr.to_string(),
            proxy_client.external_client_addr.to_string()
        )
        .as_str(),
        "core::tunnel::proxy::tunnel_client_proxy",
    )
    .await;
    Ok(())
}
