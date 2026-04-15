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
use crate::orm::tunnel_session::new_tunnel_session;
use crate::orm::tunnel_user::authenticate_tunnel_user;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::{mpsc, watch};

pub async fn tunnel_client_control(
    flags: Flags,
    shared: Shared,
    tunnel_client: Arc<TunnelClient>,
    tunnel_status: Arc<TunnelStatus>,
) {
    let mut client_type: Option<ClientType> = None;
    let mut buffer = vec![0u8; 1024];
    let (heartbeat_tx, heartbeat_rx) = watch::channel(false);
    let (control_tx, control_rx) = mpsc::channel::<Message>(1);
    let mut control_rx = Some(control_rx);
    let control_message_sender_client = ControlMessageSenderClient::new(control_tx);

    let mut tunnel_client_heartbeat_thread = None;
    let mut tunnel_client_proxy_control_thread = None;
    let mut tunnel_client_proxy_thread = None;
    let mut tunnel_control_message_sender_thread = None;

    loop {
        let read_future = async {
            let mut guard = tunnel_client.stream_rx.lock().await;
            read_message(&mut guard, &mut buffer).await
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
            result = read_future => {
                let Ok(message) = result else {
                    if tunnel_control_message_sender_thread.is_some() {
                        handle_bad_request_handler(flags.clone(), tunnel_client.addr, control_message_sender_client.clone()).await;
                    } else {
                        handle_bad_request_stream(flags.clone(), tunnel_client.clone()).await;
                    }
                    break;
                };
                match message.message_type {
                    MessageType::Heartbeat => {
                        log(Level::Debug, "Heartbeat", "tunnel_client_control").await;
                        heartbeat_tx.send_replace(true);
                    },
                    MessageType::Service => {
                        let Some(control_rx) = control_rx.take() else {
                            handle_bad_request_stream(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };

                        if client_type.is_some() {
                            handle_bad_request_handler(flags.clone(), tunnel_client.addr, control_message_sender_client.clone()).await;
                            break;
                        }

                        tunnel_control_message_sender_thread = Some(tokio::spawn(
                            tunnel_control_message_sender(
                                flags.clone(),
                                control_rx,
                                tunnel_client.clone()
                            )
                        ));

                        let Ok(service_message) = serde_json::from_str::<ServiceMessage>(message.message_string.as_str()) else {
                            handle_bad_request_handler(flags.clone(), tunnel_client.addr, control_message_sender_client).await;
                            break;
                        };

                        let user_id = match service_message.auth {
                            ServiceAuth::Token { token } => {
                                //  TODO token verification
                                //  TODO return user_id
                                todo!()
                            },

                            ServiceAuth::Password { username, password } => {
                                authenticate_tunnel_user(
                                    shared.clone(),
                                    username.as_str(),
                                    password.as_str()
                                )
                                .await
                                .unwrap_or(None)
                            },
                        };

                        let Some(user_id) = user_id else {
                            log(
                                Level::Notice,
                                format!("Access from {} denied", tunnel_client.addr.to_string()).as_str(),
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
                                tunnel_client.clone(),
                                tunnel_status.clone(),
                                control_message_sender_client.clone()
                            ))
                        );
                    }
                    MessageType::Proxy => {
                        if client_type.is_some() {
                            handle_bad_request_stream(flags.clone(), tunnel_client.clone()).await;
                            break;
                        }

                        let Ok(client_info) = serde_json::from_str::<ProxyMessage>(message.message_string.as_str()) else {
                            handle_bad_request_stream(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };
                        let Some(proxy_client) = tunnel_status.proxy_queue.write().await.remove(&client_info.proxy_id) else {
                            handle_bad_request_stream(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };
                        client_type = Some(ClientType::Proxy);
                        if let Err(error) = new_tunnel_session(
                            shared.clone(),
                            proxy_client.proxy_id.clone(),
                            proxy_client.tunnel_client_user_id.clone(),
                            tunnel_client.addr,
                            proxy_client.external_client_addr
                        ).await {
                            log(
                                Level::Warning,
                                format!("Unable to update database: {:?}", error).as_str(),
                                "tunnel::control::tunnel_client_control"
                            )
                            .await;
                        }
                        tunnel_client_proxy_thread = Some(
                            tokio::spawn(tunnel_client_proxy(
                                flags.clone(),
                                shared.clone(),
                                tunnel_client.clone(),
                                proxy_client,
                                tunnel_status.clone()
                            ))
                        );
                        break;
                    }
                    MessageType::Port => {
                        //  does not occur under normal circumstances
                        flags.local_cancellation_token.cancel();
                        break;
                    },
                    MessageType::Close => {
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                    MessageType::Empty => {
                        //  placeholder message type
                    }
                    MessageType::Error => {
                        log(
                            Level::Info,
                            format!(
                                "Connection with client {} closed with an error: {}",
                                tunnel_client.addr.to_string() ,
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

    let _shutdown_status = tunnel_client.stream_tx.lock().await.shutdown().await;

    log(
        Level::Info,
        format!("Connection with {} closed", tunnel_client.addr.to_string()).as_str(),
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

async fn handle_bad_request_stream(flags: Flags, tunnel_client: Arc<TunnelClient>) {
    log(
        Level::Debug,
        format!("Bad request from {}", tunnel_client.addr.to_string()).as_str(),
        "core::tunnel::control::tunnel_client_control",
    )
    .await;

    let message = Message::new(MessageType::Error, "bad request".to_string());

    let _res = send_message(&mut *tunnel_client.stream_tx.lock().await, &message).await;

    flags.local_cancellation_token.cancel();
}
