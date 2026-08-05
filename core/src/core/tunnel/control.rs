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
use crate::config::tunnel::TUNNEL_CLIENT_HEARTBEAT_TIMEOUT;
use crate::core::message::common::{MessageBuilder, MessageParser};
use crate::core::message::message::{
    Message, MessageType, ProxyMessage, ServiceAuth, ServiceMessage,
};
use crate::core::message::v1::common::MESSAGE_VERSION_V1;
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::message_handler::{
    ControlMessageSenderClient, tunnel_control_message_sender,
};
use crate::core::tunnel::model::{ClientType, Flags, TunnelClient, TunnelStatus};
use crate::core::tunnel::proxy::{tunnel_client_proxy, tunnel_client_proxy_control};
use crate::orm::tunnel_session::DatabaseTunnelSessionAction;
use crate::orm::tunnel_user::{authenticate_tunnel_token, authenticate_tunnel_user};
use axum::body::Bytes;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::{Semaphore, mpsc, watch};
use tokio::time::{Instant, sleep_until};
use tokio_rustls::server::TlsStream;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::future::FutureExt;
use tracing::{debug, info, warn};

pub async fn tunnel_client_control(
    flags: Flags,
    shared: Shared,
    tunnel_client_stream: TlsStream<TcpStream>,
    tunnel_client_addr: SocketAddr,
    tunnel_status: Arc<TunnelStatus>,
    tunnel_info: Arc<TunnelInfo>,
    tunnel_global_connection_semaphore: Arc<Semaphore>,
) {
    let mut client_type: Option<ClientType> = None;

    let (framed_writer, framed_reader) = LengthDelimitedCodec::builder()
        .length_field_offset(0)
        .length_field_type::<u8>()
        .length_adjustment(0)
        .new_framed(tunnel_client_stream)
        .split();
    let mut tunnel_client_writer = Some(framed_writer);
    let mut tunnel_client_reader = Some(framed_reader);

    let (heartbeat_tx, heartbeat_rx) = watch::channel(false);
    let (control_tx, control_rx) = mpsc::channel::<Message>(1024);
    let mut control_rx = Some(control_rx);
    let control_message_sender_client =
        ControlMessageSenderClient::new(control_tx, MESSAGE_VERSION_V1);

    let mut authentication_timeout = Some(Instant::now() + Duration::from_millis(5000));

    let mut tunnel_client_heartbeat_task = None;
    let mut tunnel_client_proxy_control_task = None;
    let mut tunnel_client_proxy_task = None;
    let mut tunnel_control_message_sender_task = None;

    loop {
        let read_future = async {
            let Some(tunnel_client_reader_ref) = tunnel_client_reader.as_mut() else {
                unreachable!(); //  This thread cannot not be reading if ownership is transferred
            };
            let read_result = tunnel_client_reader_ref.next().await;
            match read_result {
                Some(Ok(mut bytes_read)) => match MessageParser::parse(bytes_read.split().freeze())
                {
                    Ok(message) => Ok(message),
                    Err(error) => Err(error.into()),
                },
                Some(Err(error)) => Err(error.into()),
                None => Err(TunnelError::ClientClosed),
            }
        };

        select! {
            biased;
            _ = flags.local_cancellation_token.cancelled() => {
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
                            handle_bad_request_handler(flags.clone(), control_message_sender_client).await;
                        }
                        //  ClientType::Proxy impossible because the Proxy branch breaks out of the loop
                        None => {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_writer).await;
                        }
                    }
                    break;
                };
                match message.message_type {
                    MessageType::Heartbeat => {
                        debug!("Heartbeat received");
                        heartbeat_tx.send_replace(true);
                    },
                    MessageType::Service => {
                        //  check if client is already a service connection
                        if client_guard(client_type, flags.clone(), control_message_sender_client.clone()).await.is_err() {
                            break;
                        }

                        //  ownership
                        let Some(control_rx) = control_rx.take() else {
                            unreachable!(); //  second control request
                        };

                        let Some(tunnel_client_tx) = tunnel_client_writer.take() else {
                            unreachable!(); //  tunnel_client_tx only taken when client_type is set
                        };

                        //  unified sender for stream connection
                        tunnel_control_message_sender_task = Some(tokio::spawn(
                            tunnel_control_message_sender(
                                flags.clone(),
                                control_rx,
                                tunnel_client_tx,
                            )
                        ));

                        //  validation and parsing
                        let Ok(payload_str) = str::from_utf8(&message.message_payload) else {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_writer).await;
                            break;
                        };

                        let Ok(service_message) = serde_json::from_str::<ServiceMessage>(payload_str) else {
                            handle_bad_request_handler(flags.clone(), control_message_sender_client).await;
                            break;
                        };

                        authentication_timeout = None;

                        //  authentication
                        let user_id = match service_message.auth {
                            ServiceAuth::Token { token } => {
                                authenticate_tunnel_token(
                                    shared.db_connection.clone(),
                                    token.as_str()
                                )
                                .await
                            },

                            ServiceAuth::Password { username, password } => {
                                match authenticate_tunnel_user(shared.db_connection.clone(),
                                    shared.auth_manager.clone(),
                                    username.as_str(),
                                    password.as_str()
                                )
                                .await {
                                    Ok((id, _)) => Ok(id),
                                    Err(error) => Err(error),
                                }

                            },
                        };

                        let Ok(user_id) = user_id else {
                            warn!("Access denied");

                            let _ = control_message_sender_client.send_message(MessageType::Error, "access denied").await;

                            flags.local_cancellation_token.cancel();
                            break;
                        };

                        client_type = Some(ClientType::Service);

                        //  spawn tasks
                        tunnel_client_heartbeat_task = Some(
                            tokio::spawn(tunnel_client_heartbeat(
                                flags.clone(),
                                control_message_sender_client.clone(),
                                (heartbeat_tx.clone(), heartbeat_rx.clone())
                            ))
                        );
                        tunnel_client_proxy_control_task = Some(
                            tokio::spawn(tunnel_client_proxy_control(
                                flags.clone(),
                                user_id,
                                tunnel_status.clone(),
                                tunnel_info.clone(),
                                control_message_sender_client.clone(),
                                tunnel_global_connection_semaphore.clone()
                            ))
                        );
                    }
                    MessageType::Proxy => {
                        //  check if client is already a service connection
                        if client_guard(client_type, flags.clone(), control_message_sender_client).await.is_err() {
                            break;
                        }

                        //  validation and parsing
                        let Ok(payload_str) = str::from_utf8(&message.message_payload) else {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_writer).await;
                            break;
                        };

                        let Ok(client_info) = serde_json::from_str::<ProxyMessage>(payload_str) else {
                            handle_bad_request_stream(flags.clone(), &mut tunnel_client_writer).await;
                            break;
                        };

                        //  get external connection with proxy id
                        let Some((_, proxy_client)) = tunnel_status.pending_external_clients.remove(&client_info.proxy_id) else {
                            warn!("Proxy access denied");

                            let Some(tunnel_client_tx) = tunnel_client_writer.as_mut() else {
                                unreachable!("tunnel_client_writer only taken when client_type is set");
                            };

                            let message = Message::new(MESSAGE_VERSION_V1, MessageType::Error, "access denied");
                            let mut write_buffer = BytesMut::with_capacity(256);
                            if let Err(error) = MessageBuilder::encode(&message, &mut write_buffer) {
                                warn!("Unable to encode message: {:?}", error);
                                return;
                            }

                            let _ = tunnel_client_tx
                                .send(write_buffer.split().freeze())
                                .with_cancellation_token_owned(flags.local_cancellation_token.clone())
                                .await;

                            flags.local_cancellation_token.cancel();
                            break;
                        };

                        //  ownership
                        let Some(tunnel_client_writer) = tunnel_client_writer.take() else {
                            unreachable!("tunnel_client_tx only taken when client_type is set");
                        };
                        let Some(tunnel_client_reader) = tunnel_client_reader.take() else {
                            unreachable!("tunnel_client_rx only taken when client_type is set");
                        };

                        //  proxy branch breaks out of the loop, no need to assign
                        // authentication_timeout = None;
                        // client_type = Some(ClientType::Proxy);

                        //  database
                        if let Err(error) = shared.database_tunnel_session_batch_tx.send(
                            DatabaseTunnelSessionAction::Update {
                                user_id: proxy_client.tunnel_client_user_id.clone(),
                                tunnel_client: tunnel_client_addr.ip(),
                                inbound: 0,
                                outbound: 0,
                                external_connection_count_update: true
                            }
                        )
                        .await {
                            warn!("Unable to insert into database: {:?}", error);
                        }

                        //  spawn tasks
                        tunnel_client_proxy_task = Some(
                            tokio::spawn(tunnel_client_proxy(
                                flags.clone(),
                                shared.clone(),
                                TunnelClient {
                                    stream: tunnel_client_writer.reunite(tunnel_client_reader).expect("`tunnel_client_writer` and `tunnel_client_reader` must be corresponding halves").into_inner(),
                                    addr: tunnel_client_addr
                                },
                                tunnel_info,
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
                        info!("Session ended with error: {}", str::from_utf8(&message.message_payload).unwrap_or("Invalid error payload"));
                        flags.local_cancellation_token.cancel();
                        break;
                    }
                }
            },
        }
    }

    if let Some(task) = tunnel_client_proxy_task {
        let _ = task.await;
    }

    if let Some(task) = tunnel_client_heartbeat_task {
        let _ = task.await;
    }

    if let Some(task) = tunnel_client_proxy_control_task {
        let _ = task.await;
    }

    if let Some(task) = tunnel_control_message_sender_task {
        let _ = task.await;
    }

    if let (Some(tunnel_client_writer), Some(tunnel_client_reader)) =
        (tunnel_client_writer, tunnel_client_reader)
    {
        let _ = tunnel_client_writer
            .reunite(tunnel_client_reader)
            .expect(
                "`tunnel_client_writer` and `tunnel_client_reader` must be corresponding halves",
            )
            .into_inner()
            .shutdown()
            .await;
    }

    info!("Session ended");
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
            _ = flags.local_cancellation_token.cancelled() => None,
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
                    _ = flags.local_cancellation_token.cancelled() => { break; },
                    _ = tokio::time::sleep(TUNNEL_CLIENT_HEARTBEAT_TIMEOUT) => {},
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
            .send_message(MessageType::Heartbeat, "")
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
    control_message_sender_client: ControlMessageSenderClient,
) -> Result<(), ()> {
    match client_type {
        Some(ClientType::Service) => {
            handle_bad_request_handler(flags.clone(), control_message_sender_client).await;
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
    control_message_sender_client: ControlMessageSenderClient,
) {
    warn!("Bad request");

    let _ = control_message_sender_client
        .send_message(MessageType::Error, "bad request")
        .await;

    flags.local_cancellation_token.cancel();
}

async fn handle_bad_request_stream(
    flags: Flags,
    tunnel_client_tx: &mut Option<
        SplitSink<Framed<TlsStream<TcpStream>, LengthDelimitedCodec>, Bytes>,
    >,
) {
    let Some(tunnel_client_tx) = tunnel_client_tx else {
        unreachable!();
    };

    warn!("Bad request");

    let message = Message::new(MESSAGE_VERSION_V1, MessageType::Error, "bad request");

    let mut write_buffer = BytesMut::with_capacity(256);
    if let Err(error) = MessageBuilder::encode(&message, &mut write_buffer) {
        warn!("Unable to encode message: {:?}", error);
        return;
    }

    let _ = tunnel_client_tx
        .send(write_buffer.split().freeze())
        .with_cancellation_token_owned(flags.local_cancellation_token.clone())
        .await;

    flags.local_cancellation_token.cancel();
}
