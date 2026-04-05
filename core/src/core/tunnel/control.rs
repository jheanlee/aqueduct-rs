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
use crate::config::tunnel::TUNNEL_CLIENT_HEARTBEAT_TIMEOUT;
use crate::core::message::message::{
    Message, MessageType, ProxyMessage, ServiceAuth, ServiceMessage,
};
use crate::core::socket::io::{read_message, send_message};
use crate::core::tunnel::model::{ClientType, Flags, TunnelClient, TunnelStatus};
use crate::core::tunnel::proxy::{tunnel_client_proxy, tunnel_client_proxy_control};
use crate::orm::user::authenticate_user;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::select;
use tokio::sync::watch;

pub async fn tunnel_client_control(
    flags: Flags,
    tunnel_client: Arc<TunnelClient>,
    tunnel_status: Arc<TunnelStatus>,
) {
    let mut client_type: Option<ClientType> = None;
    let mut buffer = [0u8; 1024];
    let (heartbeat_tx, heartbeat_rx) = watch::channel(false);

    let mut tunnel_client_heartbeat_thread = None;
    let mut tunnel_client_proxy_control_thread = None;
    let mut tunnel_client_proxy_thread = None;

    loop {
        let read_future = async {
            let mut guard = tunnel_client.stream_rx.lock().await;
            read_message(guard.deref_mut(), buffer.as_mut()).await
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
                    handle_bad_request(flags.clone(), tunnel_client.clone()).await;
                    break;
                };
                match message.message_type {
                    MessageType::Heartbeat => {
                        log(Level::Debug, "Heartbeat", "tunnel_client_control").await;
                        heartbeat_tx.send_replace(true);
                    },
                    MessageType::Service => {
                        let Ok(service_message) = serde_json::from_str::<ServiceMessage>(message.message_string.as_str()) else {
                            handle_bad_request(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };

                        let authorized = match service_message.auth {
                            ServiceAuth::Token { token } => {
                                // TODO token verification
                                true
                            },

                            ServiceAuth::Password { username, password } => {
                                authenticate_user(
                                    &tunnel_status.db_connection,
                                    username.as_str(),
                                    password.as_str()
                                )
                                .await
                                .unwrap_or(false)
                            },
                        };

                        if authorized {
                            client_type = Some(ClientType::Service);
                            tunnel_client_heartbeat_thread = Some(
                                tokio::spawn(tunnel_client_heartbeat(
                                    flags.clone(),
                                    tunnel_client.clone(),
                                    (heartbeat_tx.clone(), heartbeat_rx.clone())
                                ))
                            );
                            tunnel_client_proxy_control_thread = Some(
                                tokio::spawn(tunnel_client_proxy_control(
                                    flags.clone(),
                                    tunnel_client.clone(),
                                    tunnel_status.clone()
                                ))
                            );
                        } else {
                            log(
                                Level::Notice,
                                format!("Access from {} denied", tunnel_client.addr.to_string()).as_str(),
                                "core::tunnel::control::tunnel_client_control"
                            )
                            .await;

                            let message = Message::new(
                                MessageType::Error,
                                "access denied".to_string()
                            );

                            let _res = send_message(
                                tunnel_client.stream_tx.lock().await.deref_mut(),
                                &message
                            )
                            .await;

                            flags.local_cancellation_token.cancel();
                            break;
                        }
                    }
                    MessageType::Proxy => {
                        let Ok(client_info) = serde_json::from_str::<ProxyMessage>(message.message_string.as_str()) else {
                            handle_bad_request(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };
                        let Some(proxy_client) = tunnel_status.proxy_queue.write().await.remove(&client_info.proxy_id) else {
                            handle_bad_request(flags.clone(), tunnel_client.clone()).await;
                            break;
                        };
                        client_type = Some(ClientType::Proxy);
                        tunnel_client_proxy_thread = Some(
                            tokio::spawn(tunnel_client_proxy(
                                flags.clone(),
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
                    MessageType::Error => {
                        log(Level::Info, format!("Connection with client {} closed with an error: {}", tunnel_client.addr.to_string() ,message.message_string).as_str(), "tunnel::control::tunnel_client_control").await;
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
    tunnel_client: Arc<TunnelClient>,
    (heartbeat_tx, mut heartbeat_rx): (watch::Sender<bool>, watch::Receiver<bool>),
) {
    let message = Message::new(MessageType::Heartbeat, String::new());

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
        let write_future = async {
            let mut guard = tunnel_client.stream_tx.lock().await;
            heartbeat_tx.send_replace(false);
            heartbeat_rx.borrow_and_update();
            send_message(guard.deref_mut(), &message).await
        };

        select! {
            biased;
            _global_cancalled = flags.global_cancellation_token.cancelled() => { break; },
            _client_cancealled = flags.local_cancellation_token.cancelled() => { break; },
            write_result = write_future => {
                if let Err(error) = write_result {
                    log(Level::Debug, format!("Unable to send heartbeat to {}: {:?}", tunnel_client.addr.to_string(), error).as_str(), "core::tunnel::control::tunnel_client_heartbeat").await;
                    flags.local_cancellation_token.cancel();
                    break;
                }
            },
        }
    }
}

async fn handle_bad_request(flags: Flags, tunnel_client: Arc<TunnelClient>) {
    log(
        Level::Debug,
        format!("Bad request from {}", tunnel_client.addr.to_string()).as_str(),
        "core::tunnel::control::tunnel_client_control",
    )
    .await;

    let message = Message::new(MessageType::Error, "bad request".to_string());

    let _res = send_message(tunnel_client.stream_tx.lock().await.deref_mut(), &message).await;

    flags.local_cancellation_token.cancel();
}
