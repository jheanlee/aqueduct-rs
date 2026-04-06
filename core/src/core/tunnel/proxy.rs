/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::common::log::{Level, log};
use crate::core::message::message::MessageType;
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::error::TunnelError::NoPortsAvailable;
use crate::core::tunnel::message_handler::ControlMessageSenderClient;
use crate::core::tunnel::model::{Flags, ProxyClient, TunnelClient, TunnelStatus};
use nanoid::nanoid;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::select;

pub async fn tunnel_client_proxy_control(
    flags: Flags,
    tunnel_client: Arc<TunnelClient>,
    tunnel_status: Arc<TunnelStatus>,
    control_message_sender_client: ControlMessageSenderClient,
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
        let _ = control_message_sender_client
            .send_message(MessageType::Error, "no ports available".to_string())
            .await;
        flags.local_cancellation_token.cancel();
        Err(NoPortsAvailable)?
    };

    //  send port number to client
    let port = tcp_listener.local_addr()?.port();
    if control_message_sender_client
        .send_message(MessageType::Port, format!("{port}"))
        .await
        .is_err()
    {
        flags.local_cancellation_token.cancel();
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
                        {
                            //  insert into queue
                            let (external_client_stream_rx, external_client_stream_tx) = external_client_stream.into_split();
                            let mut proxy_queue = tunnel_status.proxy_queue.write().await;
                            proxy_queue.insert(
                                id.clone(),
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

                        //  notify client of the new user
                        if control_message_sender_client.send_message(MessageType::Proxy, id.clone()).await.is_err() {
                            flags.local_cancellation_token.cancel();
                            break;
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

    let mut available_ports = tunnel_status.available_ports.write().await;
    available_ports.push_back(port);

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
        select! {
            tunnel_client_read = tunnel_client_stream_rx.read(&mut tunnel_buffer) => {
                //  client (service) -> external_client
                match tunnel_client_read {
                    Ok(0) => { break; }
                    Ok(bytes_read) => {
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
                    Ok(0) => { break; }
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
