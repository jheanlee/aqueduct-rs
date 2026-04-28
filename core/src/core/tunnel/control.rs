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
use crate::config::tunnel::TUNNEL_CLIENT_HEARTBEAT_TIMEOUT;
use crate::core::message::message::{
    Message, MessageType, ProxyMessage, ServiceAuth, ServiceMessage,
};
use crate::core::socket::io::{read_message, send_message};
use crate::core::tunnel::message_handler::{
    ControlMessageSenderClient, tunnel_control_message_sender,
};
use crate::core::tunnel::model::{ClientType, Flags, TunnelClient, TunnelStatus};
use crate::core::tunnel::proxy::{tunnel_client_proxy, tunnel_client_proxy_control};
use crate::orm::tunnel_session::DatabaseTunnelSessionAction;
use crate::orm::tunnel_user::{authenticate_tunnel_token, authenticate_tunnel_user};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Semaphore, mpsc, watch};
use tokio::time::{Instant, sleep_until};
use tokio::{io, select};
use tokio_rustls::server::TlsStream;

pub async fn tunnel_client_control(
    flags: Flags,
    shared: Shared,
    tunnel_client_stream: TlsStream<TcpStream>,
    tunnel_client_addr: SocketAddr,
    tunnel_status: Arc<TunnelStatus>,
    tunnel_global_connection_semaphore: Arc<Semaphore>,
) {
    let mut client_type: Option<ClientType> = None;

    let (tunnel_client_rx, tunnel_client_tx) = io::split(tunnel_client_stream);
    let mut tunnel_client_tx = Some(tunnel_client_tx);
    let mut tunnel_client_rx = Some(tunnel_client_rx);

    let (heartbeat_tx, heartbeat_rx) = watch::channel(false);
    let (control_tx, control_rx) = mpsc::channel::<Message>(1024);
    let mut control_rx = Some(control_rx);
    let control_message_sender_client = ControlMessageSenderClient::new(control_tx);

    let mut authentication_timeout = Some(Instant::now() + Duration::from_millis(5000));

    let mut tunnel_client_heartbeat_thread = None;
    let mut tunnel_client_proxy_control_thread = None;
    let mut tunnel_client_proxy_thread = None;
    let mut tunnel_control_message_sender_thread = None;

    loop {
        let read_future = async {
            let Some(tunnel_client_rx_ref) = tunnel_client_rx.as_mut() else {
                unreachable!(); //  This thread cannot not be reading if ownership is transferred
            };
            read_message(tunnel_client_rx_ref).await
        };

        select! {
            biased;
            _global_cancalled = flags.global_cancellation_token.cancelled() => {
                flags.local_cancellation_token.cancel();
                break;
            },
            _client_cancealled = flags.local_cancellation_token.cancelled() => {
                break;
            },
            _auth_timedout = sleep_until(if authentication_timeout.is_some() { authentication_timeout.unwrap() } else { Instant::now() + Duration::from_hours(10000) }), if authentication_timeout.is_some() => {
                flags.local_cancellation_token.cancel();
                break;
            },
            result = read_future => {
                let Ok(message) = result else {
                    match client_type {
                        Some(ClientType::Service) => {
                            handle_bad_request_handler(flags.clone(), tunnel_client_addr, control_message_sender_client).await;
                        }
                        // Some(ClientType::Proxy) => {
                        //     log(
                        //         Level::Debug,
                        //         format!("Bad request from {}", tunnel_client_addr.to_string()).as_str(),
                        //         "core::tunnel::control::tunnel_client_control",
                        //     )
                        //     .await;
                        //     flags.local_cancellation_token.cancel();
                        // }
                        None => {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_tx, tunnel_client_addr).await;
                        }
                    }
                    break;
                };
                match message.message_type {
                    MessageType::Heartbeat => {
                        log(Level::Debug, "Heartbeat", "tunnel_client_control").await;
                        heartbeat_tx.send_replace(true);
                    },
                    MessageType::Service => {
                        if client_guard(client_type, flags.clone(), tunnel_client_addr, control_message_sender_client.clone()).await.is_err() {
                            break;
                        }

                        let Some(control_rx) = control_rx.take() else {
                            unreachable!(); //  second control request
                        };

                        let Some(tunnel_client_tx) = tunnel_client_tx.take() else {
                            unreachable!(); //  tunnel_client_tx only taken when client_type is set
                        };

                        tunnel_control_message_sender_thread = Some(tokio::spawn(
                            tunnel_control_message_sender(
                                flags.clone(),
                                control_rx,
                                tunnel_client_tx,
                                tunnel_client_addr
                            )
                        ));

                        let Ok(service_message) = serde_json::from_str::<ServiceMessage>(message.message_string.as_str()) else {
                            handle_bad_request_handler(flags.clone(), tunnel_client_addr, control_message_sender_client).await;
                            break;
                        };

                        authentication_timeout = None;

                        let user_id = match service_message.auth {
                            ServiceAuth::Token { token } => {
                                authenticate_tunnel_token(shared.clone(), token.as_str()).await
                            },

                            ServiceAuth::Password { username, password } => {
                                authenticate_tunnel_user(
                                    shared.clone(),
                                    username.as_str(),
                                    password.as_str()
                                )
                                .await
                            },
                        };

                        let Ok(user_id) = user_id else {
                            log(
                                Level::Notice,
                                format!("Access from {} denied", tunnel_client_addr.to_string()).as_str(),
                                "core::tunnel::control::tunnel_client_control"
                            )
                            .await;

                            let _ = control_message_sender_client.send_message(MessageType::Error, "access denied".to_string()).await;

                            flags.local_cancellation_token.cancel();
                            break;
                        };

                        client_type = Some(ClientType::Service);
                        tunnel_client_heartbeat_thread = Some(
                            tokio::spawn(tunnel_client_heartbeat(
                                flags.clone(),
                                control_message_sender_client.clone(),
                                (heartbeat_tx.clone(), heartbeat_rx.clone())
                            ))
                        );
                        tunnel_client_proxy_control_thread = Some(
                            tokio::spawn(tunnel_client_proxy_control(
                                flags.clone(),
                                user_id,
                                tunnel_client_addr,
                                tunnel_status.clone(),
                                control_message_sender_client.clone(),
                                tunnel_global_connection_semaphore.clone()
                            ))
                        );
                    }
                    MessageType::Proxy => {
                        if client_guard(client_type, flags.clone(), tunnel_client_addr, control_message_sender_client).await.is_err() {
                            break;
                        }

                        let Ok(client_info) = serde_json::from_str::<ProxyMessage>(message.message_string.as_str()) else {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_tx, tunnel_client_addr).await;
                            break;
                        };
                        let Some((_, proxy_client)) = tunnel_status.pending_external_clients.remove(&client_info.proxy_id) else {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_tx, tunnel_client_addr).await;
                            break;
                        };

                        let Some(tunnel_client_tx) = tunnel_client_tx.take() else {
                            unreachable!("tunnel_client_tx only taken when client_type is set");
                        };
                        let Some(tunnel_client_rx) = tunnel_client_rx.take() else {
                            unreachable!("tunnel_client_rx only taken when client_type is set");
                        };

                        // //  breaks out of the loop, no need to assign
                        // authentication_timeout = None;
                        // client_type = Some(ClientType::Proxy);

                        if let Err(error) = shared.database_tunnel_session_batch_tx.send(
                            DatabaseTunnelSessionAction::Update {
                                user_id: proxy_client.tunnel_client_user_id.clone(),
                                tunnel_client: tunnel_client_addr.ip().to_string(),
                                inbound: 0,
                                outbound: 0,
                                external_connection_count_update: true
                            }
                        )
                        .await {
                            log(
                                Level::Warning,
                                format!("Unable to insert into database: {:?}", error).as_str(),
                                "tunnel::control::tunnel_client_control"
                            ).await;
                        }

                        tunnel_client_proxy_thread = Some(
                            tokio::spawn(tunnel_client_proxy(
                                flags.clone(),
                                shared.clone(),
                                TunnelClient {
                                    stream_tx: tunnel_client_tx,
                                    stream_rx: tunnel_client_rx,
                                    addr: tunnel_client_addr
                                },
                                proxy_client,
                            ))
                        );
                        break;
                    }
                    MessageType::Close => {
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                    MessageType::Empty => {
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                    MessageType::Error => {
                        log(
                            Level::Info,
                            format!(
                                "Connection with client {} closed with an error: {}",
                                tunnel_client_addr.to_string(),
                                message.message_string
                            )
                            .as_str(),
                            "tunnel::control::tunnel_client_control"
                        )
                        .await;
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                }
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

    if let Some(thread) = tunnel_control_message_sender_thread {
        let _ = thread.await;
    }

    if let Some(mut tunnel_client_tx) = tunnel_client_tx {
        let _shutdown_status = tunnel_client_tx.shutdown().await;
    }

    log(
        Level::Info,
        format!("Connection with {} closed", tunnel_client_addr.to_string()).as_str(),
        "core::tunnel::control::tunnel_client_control",
    )
    .await;
}

pub async fn tunnel_client_heartbeat(
    flags: Flags,
    control_message_sender_client: ControlMessageSenderClient,
    (heartbeat_tx, mut heartbeat_rx): (watch::Sender<bool>, watch::Receiver<bool>),
) {
    loop {
        //  wait for heartbeat
        let value = select! {
            biased;
            _global_cancalled = flags.global_cancellation_token.cancelled() => None,
            _client_cancealled = flags.local_cancellation_token.cancelled() => None,
            heartbeat_changed = heartbeat_rx.changed() => {
                if heartbeat_changed.is_ok() {
                    Some(*heartbeat_rx.borrow())
                } else {
                    flags.local_cancellation_token.cancel();
                    None
                }
            },
            _sleep = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => None
        };

        //  sleep until next cycle
        match value {
            Some(value) if value => {
                select! {
                    biased;
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
        heartbeat_tx.send_replace(false);
        heartbeat_rx.borrow_and_update();
        if control_message_sender_client
            .send_message(MessageType::Heartbeat, String::new())
            .await
            .is_err()
        {
            flags.local_cancellation_token.cancel();
            break;
        }
    }
}

async fn client_guard(
    client_type: Option<ClientType>,
    flags: Flags,
    tunnel_client_addr: SocketAddr,
    control_message_sender_client: ControlMessageSenderClient,
) -> Result<(), ()> {
    match client_type {
        Some(ClientType::Service) => {
            handle_bad_request_handler(
                flags.clone(),
                tunnel_client_addr,
                control_message_sender_client,
            )
            .await;
            Err(())
        }
        // Some(ClientType::Proxy) => {
        //     log(
        //         Level::Debug,
        //         format!("Bad request from {}", tunnel_client_addr.to_string()).as_str(),
        //         "core::tunnel::control::tunnel_client_control",
        //     )
        //     .await;
        //     flags.local_cancellation_token.cancel();
        //     Err(())
        // }
        None => Ok(()),
    }
}

async fn handle_bad_request_handler(
    flags: Flags,
    tunnel_client_addr: SocketAddr,
    control_message_sender_client: ControlMessageSenderClient,
) {
    log(
        Level::Debug,
        format!("Bad request from {}", tunnel_client_addr.to_string()).as_str(),
        "core::tunnel::control::tunnel_client_control",
    )
    .await;

    let _ = control_message_sender_client
        .send_message(MessageType::Error, "bad request".to_string())
        .await;

    flags.local_cancellation_token.cancel();
}

async fn handle_bad_request_stream(
    flags: Flags,
    tunnel_client_tx: &mut Option<WriteHalf<TlsStream<TcpStream>>>,
    tunnel_client_addr: SocketAddr,
) {
    let Some(tunnel_client_tx) = tunnel_client_tx else {
        unreachable!();
    };

    log(
        Level::Debug,
        format!("Bad request from {}", tunnel_client_addr.to_string()).as_str(),
        "core::tunnel::control::tunnel_client_control",
    )
    .await;

    let message = Message::new(MessageType::Error, "bad request".to_string());

    let _res = send_message(tunnel_client_tx, &message).await;

    flags.local_cancellation_token.cancel();
}
