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
use crate::core::message::common::MessageBuilder;
use crate::core::message::types::{Message, MessageType};
use crate::core::tunnel::error::TunnelError;
use futures::SinkExt;
use futures::stream::SplitSink;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_rustls::server::TlsStream;
use tokio_util::bytes::{Bytes, BytesMut};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tokio_util::future::FutureExt;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct ControlMessageSenderClient {
    pub addr: SocketAddr,
    control_message_handler_tx: mpsc::Sender<Message>,
    message_version: u8,
}

impl ControlMessageSenderClient {
    pub fn new(addr: SocketAddr, tx: mpsc::Sender<Message>, message_version: u8) -> Self {
        ControlMessageSenderClient {
            addr,
            control_message_handler_tx: tx,
            message_version,
        }
    }

    pub async fn send_message(
        &self,
        message_type: MessageType,
        message_payload: &str,
    ) -> Result<(), TunnelError> {
        self.control_message_handler_tx
            .send(Message::new(
                self.message_version,
                message_type,
                message_payload,
            ))
            .await?;
        Ok(())
    }
}

pub async fn tunnel_control_message_sender(
    mut control_rx: mpsc::Receiver<Message>,
    mut tunnel_client_tx: SplitSink<Framed<TlsStream<TcpStream>, LengthDelimitedCodec>, Bytes>,
    cancellation_token: CancellationToken,
) {
    let mut write_buffer = BytesMut::with_capacity(256);

    while let Some(message) = select! {
        biased;
        message = control_rx.recv() => message,
        _ = cancellation_token.cancelled() => None
    } {
        if let Err(error) = MessageBuilder::encode(&message, &mut write_buffer) {
            warn!("Unable to encode message: {:?}", error);
            continue;
        }

        let result = match message.message_type {
            MessageType::Error | MessageType::Close => timeout(
                Duration::from_millis(1000),
                tunnel_client_tx.send(write_buffer.split().freeze()),
            )
            .await
            .ok(),
            _ => {
                tunnel_client_tx
                    .send(write_buffer.split().freeze())
                    .with_cancellation_token(&cancellation_token)
                    .await
            }
        };

        match result {
            Some(Ok(_)) => {}
            Some(Err(error)) => {
                debug!("Unable to send message to client: {:?}", error);
                break;
            }
            None => {
                break;
            }
        }
    }

    cancellation_token.cancel();
}
