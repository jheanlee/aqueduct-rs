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
use crate::common::model::Shared;
use crate::core::message::message::{ClientServiceMessage, MessageType};
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::error::TunnelError::NoPortsAvailable;
use crate::core::tunnel::message_handler::ControlMessageSenderClient;
use crate::core::tunnel::model::{Flags, ProxyClient, TunnelClient, TunnelStatus};
use crate::orm::tunnel_session::DatabaseTunnelSessionAction;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, KeyInit, Mac};
use nanoid::nanoid;
use rand::{RngExt, rng};
use serde_json::to_string;
use sha2::Sha256;
use socket2::{SockRef, TcpKeepalive};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until};

pub async fn tunnel_client_proxy_control(
    flags: Flags,
    tunnel_client_user_id: String,
    tunnel_client_addr: SocketAddr,
    tunnel_status: Arc<TunnelStatus>,
    control_message_sender_client: ControlMessageSenderClient,
    tunnel_global_connection_semaphore: Arc<Semaphore>,
) -> Result<(), TunnelError> {
    //  permit
    let client_semaphore = Arc::new(Semaphore::new(
        tunnel_status.client_connection_limit as usize,
    ));
    let mut pending_permit_threads = JoinSet::new();

    //  crypto
    let mut client_secret = [0u8; 32];
    rng().fill(&mut client_secret);

    //  assign a port
    let mut tcp_listener = None;
    {
        let mut available_ports = tunnel_status.available_ports.write().await;
        let mut port_count = available_ports.len();

        while let Some(new_port) = available_ports.pop_front()
            && port_count > 0
        {
            port_count -= 1;
            let Ok(new_tcp_listener) =
                TcpListener::bind(format!("{}:{}", tunnel_status.host, new_port)).await
            else {
                available_ports.push_back(new_port);
                continue;
            };
            tcp_listener = Some(new_tcp_listener);
            break;
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

    //  send port number and secret to client
    let port = tcp_listener.local_addr()?.port();
    if control_message_sender_client
        .send_message(
            MessageType::Service,
            to_string(&ClientServiceMessage {
                port,
                secret: BASE64_STANDARD.encode(client_secret),
            })
            .unwrap_or_else(|_| unreachable!()),
        )
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
            tunnel_client_addr.to_string()
        )
        .as_str(),
        "core::tunnel::proxy::tunnel_client_proxy_control",
    )
    .await;

    //  accept external connections
    loop {
        select! {
            _ = pending_permit_threads.join_next(), if !pending_permit_threads.is_empty() => {}
            result = tcp_listener.accept() => {
                match result {
                    Ok((external_client_stream, external_client_addr)) => {
                        if pending_permit_threads.len() >= tunnel_status.client_connection_limit as usize * 2 {
                            drop(external_client_stream);
                            log(
                                Level::Warning,
                                "External client pending exceeds double the client limit; new connections will not be accepted until available",
                                "core::tunnel::proxy::tunnel_client_proxy_control"
                            ).await;
                            continue;
                        }

                        let tunnel_status_clone = tunnel_status.clone();
                        let local_cancellation_token_clone = flags.local_cancellation_token.clone();
                        let global_semaphore_clone = tunnel_global_connection_semaphore.clone();
                        let client_semaphore_clone = client_semaphore.clone();
                        let tunnel_client_user_id_clone = tunnel_client_user_id.clone();
                        let control_message_sender_client_clone = control_message_sender_client.clone();

                        pending_permit_threads.spawn(async move {
                            let global_permit = global_semaphore_clone
                                .acquire_owned()
                                .await
                                .unwrap_or_else(|_| unreachable!("Global semaphore should not be dropped"));

                            let client_permit = client_semaphore_clone
                                .acquire_owned()
                                .await
                                .unwrap_or_else(|_| unreachable!("Client semaphore should not be dropped"));

                            let socket_ref = SockRef::from(&external_client_stream);
                            socket_ref.set_tcp_nodelay(true).unwrap_or_else(|_| { return; });
                            socket_ref.set_reuse_address(true).unwrap_or_else(|_| { return; });
                            let socket_keep_alive = TcpKeepalive::new()
                                .with_time(Duration::from_secs(60))
                                .with_interval(Duration::from_secs(30))
                                .with_retries(3);
                            socket_ref.set_tcp_keepalive(&socket_keep_alive)
                                .unwrap_or_else(|_| { return; });

                            //  generate id
                            let id = nanoid!();

                            let mut hmac: Hmac<Sha256> = Hmac::new_from_slice(&client_secret)
                                .unwrap_or_else(|_| { unreachable!("Hmac does not require key size") });
                            hmac.update(id.as_bytes());
                            let id_hash_bytes = hmac.finalize().into_bytes();
                            let id_hash = BASE64_STANDARD.encode(id_hash_bytes);

                            //  insert into queue
                            let (external_client_stream_rx, external_client_stream_tx) = external_client_stream.into_split();
                            tunnel_status_clone.pending_external_clients.insert(
                                id_hash,
                                ProxyClient {
                                    timestamp: Instant::now(),
                                    tunnel_client_user_id: tunnel_client_user_id_clone,
                                    external_client_stream_rx: external_client_stream_rx,
                                    external_client_stream_tx: external_client_stream_tx,
                                    external_client_addr: external_client_addr,
                                    _global_permit: global_permit,
                                    _client_permit: client_permit
                                }
                            );

                            //  notify client of the new user
                            if control_message_sender_client_clone.send_message(MessageType::Proxy, id.clone()).await.is_err() {
                                //  proxy clients are cleaned up by a cleaner thread if not claimed by users
                                local_cancellation_token_clone.cancel();
                            }
                        });
                    }
                    Err(error) => {
                        log(
                            Level::Warning,
                            format!("Unable to accept external connection for client {}: {:?}",
                                tunnel_client_addr.to_string(), error).as_str(),
                            "core::tunnel::proxy::tunnel_client_proxy_control"
                        ).await;
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                }
            }
            _client_cancelled = flags.local_cancellation_token.cancelled() => {
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
    shared: Shared,
    mut tunnel_client: TunnelClient,
    mut proxy_client: ProxyClient,
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

    let mut tunnel_buffer = vec![0u8; 32768];
    let mut external_buffer = vec![0u8; 32768];

    //  usage counter
    let mut inbound = 0i64;
    let mut outbound = 0i64;
    const COUNTER_UPDATE_INTERVAL: u64 = 300;
    let mut next_deadline = Instant::now() + Duration::from_secs(COUNTER_UPDATE_INTERVAL);

    loop {
        select! {
            _ = sleep_until(next_deadline) => {
                if inbound != 0 || outbound != 0 {
                    let _ = shared
                        .database_tunnel_session_batch_tx
                        .send(DatabaseTunnelSessionAction::Update {
                            user_id: proxy_client.tunnel_client_user_id.clone(),
                            tunnel_client: tunnel_client.addr.ip().to_string(),
                            inbound: inbound,
                            outbound: outbound,
                            external_connection_count_update: false,
                        })
                        .await; //  only fails when global cancellation token is set
                    inbound = 0;
                    outbound = 0;
                }
                next_deadline = Instant::now() + Duration::from_secs(COUNTER_UPDATE_INTERVAL);
            }
            tunnel_client_read = tunnel_client.stream_rx.read(&mut tunnel_buffer) => {
                //  client (service) -> external_client
                match tunnel_client_read {
                    Ok(0) => { break; }
                    Ok(bytes_read) => {
                        let write_result = proxy_client.external_client_stream_tx.write_all(&tunnel_buffer[..bytes_read]).await;
                        match write_result {
                            Ok(_) => {
                                outbound += bytes_read as i64;
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
                        let write_result = tunnel_client.stream_tx.write_all(&external_buffer[..bytes_read]).await;
                        match write_result {
                            Ok(_) => {
                                inbound += bytes_read as i64;
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

    let _ = shared
        .database_tunnel_session_batch_tx
        .send(DatabaseTunnelSessionAction::Update {
            user_id: proxy_client.tunnel_client_user_id.clone(),
            tunnel_client: tunnel_client.addr.ip().to_string(),
            inbound: inbound,
            outbound: outbound,
            external_connection_count_update: false,
        })
        .await; //  only fails when global cancellation token is set
    Ok(())
}
