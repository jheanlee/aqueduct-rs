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

use crate::common::model::Shared;
use crate::common::tunnel_info::TunnelInfo;
use crate::core::message::types::{ClientServiceMessage, MessageType};
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::error::TunnelError::NoPortsAvailable;
use crate::core::tunnel::message_handler::ControlMessageSenderClient;
use crate::core::tunnel::model::{ProxyClient, TunnelClient, TunnelStatus};
use crate::core::tunnel::proxy_io::ProxyIO;
use crate::orm::tunnel_session::DatabaseTunnelSessionAction;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::Utc;
use hmac::{Hmac, KeyInit, Mac};
use nanoid::nanoid;
use rand::{RngExt, rng};
use serde_json::to_string;
use sha2::Sha256;
use socket2::{SockRef, TcpKeepalive};
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Instant, interval};
use tokio::{io, select};
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, instrument, warn};

#[instrument(level = "info", skip_all, fields(tunnel_client = %control_message_sender_client.addr))]
pub async fn tunnel_client_proxy_control(
    tunnel_client_user_id: String,
    tunnel_status: Arc<TunnelStatus>,
    tunnel_info: Arc<TunnelInfo>,
    control_message_sender_client: ControlMessageSenderClient,
    tunnel_global_connection_semaphore: Arc<Semaphore>,
    cancellation_token: CancellationToken,
) -> Result<(), TunnelError> {
    tunnel_info
        .active_service_count
        .fetch_add(1, Ordering::Relaxed);

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
        warn!("No ports available");
        let _ = control_message_sender_client
            .send_message(MessageType::Error, "no ports available")
            .await;

        tunnel_info
            .active_service_count
            .fetch_sub(1, Ordering::Relaxed);

        cancellation_token.cancel();
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
            .unwrap_or_else(|_| unreachable!())
            .as_str(),
        )
        .await
        .is_err()
    {
        cancellation_token.cancel();
    }

    info!("Tunnel port assigned; listening on {}", port);

    //  accept external connections
    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => {
                break;
            }
            _ = pending_permit_threads.join_next(), if !pending_permit_threads.is_empty() => {}
            result = tcp_listener.accept() => {
                match result {
                    Ok((external_client_stream, external_client_addr)) => {
                        if pending_permit_threads.len() >= tunnel_status.client_connection_limit as usize * 2 {
                            drop(external_client_stream);
                            warn!("External client pending exceeds the limit; new connections will be dropped until pending queue is available");
                            continue;
                        }

                        let tunnel_status_clone = tunnel_status.clone();
                        let local_cancellation_token_clone = cancellation_token.clone();
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
                            socket_ref.set_tcp_nodelay(true).unwrap_or(());
                            socket_ref.set_reuse_address(true).unwrap_or(());
                            let socket_keep_alive = TcpKeepalive::new()
                                .with_time(Duration::from_secs(60))
                                .with_interval(Duration::from_secs(30))
                                .with_retries(3);
                            socket_ref.set_tcp_keepalive(&socket_keep_alive)
                                .unwrap_or(());

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
                                    external_client_stream_rx,
                                    external_client_stream_tx,
                                    external_client_addr,
                                    _global_permit: global_permit,
                                    _client_permit: client_permit,
                                    cancellation_token: local_cancellation_token_clone.child_token()
                                }
                            );

                            //  notify client of the new user
                            if control_message_sender_client_clone.send_message(MessageType::Proxy, id.as_str()).await.is_err() {
                                //  proxy clients are cleaned up by a cleaner thread if not claimed by users
                                local_cancellation_token_clone.cancel();
                            }
                        });
                    }
                    Err(error) => {
                        warn!("Unable to accept external connection: {:?}", error);
                        cancellation_token.cancel();
                        break;
                    }
                }
            }
        }
    }

    let mut available_ports = tunnel_status.available_ports.write().await;
    available_ports.push_back(port);

    tunnel_info
        .active_service_count
        .fetch_sub(1, Ordering::Relaxed);

    //  fd closed on drop
    Ok(())
}

#[instrument(skip_all, fields(proxy_client = %proxy_client.external_client_addr))]
pub async fn tunnel_client_proxy(
    shared: Shared,
    tunnel_client: TunnelClient,
    tunnel_info: Arc<TunnelInfo>,
    proxy_client: ProxyClient,
) -> Result<(), TunnelError> {
    debug!("TCP proxying started");
    tunnel_info
        .active_external_connection_count
        .fetch_add(1, Ordering::Relaxed);

    const BUFFER_SIZE: usize = 32768;

    //  usage counter
    let inbound = Arc::new(AtomicI64::new(0));
    let outbound = Arc::new(AtomicI64::new(0));

    const COUNTER_UPDATE_INTERVAL: u64 = 60;
    let mut timer_interval = interval(Duration::from_secs(COUNTER_UPDATE_INTERVAL));

    let (tunnel_client_rx, tunnel_client_tx) = io::split(tunnel_client.stream);

    let mut tunnel_client_io = ProxyIO::new(tunnel_client_rx, tunnel_client_tx, inbound.clone());
    let mut external_client_io = ProxyIO::new(
        proxy_client.external_client_stream_rx,
        proxy_client.external_client_stream_tx,
        outbound.clone(),
    );

    let io_copy = io::copy_bidirectional_with_sizes(
        &mut tunnel_client_io,
        &mut external_client_io,
        BUFFER_SIZE,
        BUFFER_SIZE,
    )
    .with_cancellation_token(&proxy_client.cancellation_token);

    tokio::pin!(io_copy);

    loop {
        select! {
            _ = proxy_client.cancellation_token.cancelled() => {
                break;
            }
            res = &mut io_copy => {
                match res {
                    Some(Ok(_)) => {/* gracefully closed by either service or client */}
                    Some(Err(error)) => {
                        match error.kind() {
                            ErrorKind::BrokenPipe => {
                                //  often occurs under normal circumstances
                            }
                            ErrorKind::ConnectionReset => {
                                //  often occurs under normal circumstances
                            }
                            ErrorKind::UnexpectedEof => {
                                //  often occurs under normal circumstances
                            }
                            _ => {
                                debug!("TCP proxying ended with error: {:?}", error);
                            }
                        }
                    }
                    None => { /* cancelled by cancellation token */ }
                }
                break;
            }
            _ = timer_interval.tick() => {
                let inbound = inbound.swap(0, Ordering::Relaxed);
                let outbound = outbound.swap(0, Ordering::Relaxed);
                if inbound != 0 || outbound != 0 {
                    let _ = shared
                        .database_tunnel_session_batch_tx
                        .send(DatabaseTunnelSessionAction::Update {
                            timestamp: Utc::now().naive_utc(),
                            user_id: proxy_client.tunnel_client_user_id.clone(),
                            tunnel_client: tunnel_client.addr.ip(),
                            inbound,
                            outbound,
                            external_connection_count_update: false,
                        })
                        .await; //  only fails when global cancellation token is set
                }
            }
        }
    }

    proxy_client.cancellation_token.cancel();
    debug!("TCP proxying ended");

    let _ = shared
        .database_tunnel_session_batch_tx
        .send(DatabaseTunnelSessionAction::Update {
            timestamp: Utc::now().naive_utc(),
            user_id: proxy_client.tunnel_client_user_id.clone(),
            tunnel_client: tunnel_client.addr.ip(),
            inbound: inbound.swap(0, Ordering::Relaxed),
            outbound: outbound.swap(0, Ordering::Relaxed),
            external_connection_count_update: false,
        })
        .await; //  only fails when global cancellation token is set
    tunnel_info
        .active_external_connection_count
        .fetch_sub(1, Ordering::Relaxed);
    Ok(())
}
