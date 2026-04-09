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
use crate::core::message::message::{Message, MessageType};
use crate::core::socket::io::send_message;
use crate::core::tunnel::error::TunnelError;
use crate::core::tunnel::model::{Flags, TunnelClient};
use std::sync::Arc;
use tokio::select;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct ControlMessageSenderClient {
    control_message_handler_tx: mpsc::Sender<Message>,
}

impl ControlMessageSenderClient {
    pub fn new(tx: mpsc::Sender<Message>) -> Self {
        ControlMessageSenderClient {
            control_message_handler_tx: tx,
        }
    }

    pub async fn send_message(
        &self,
        message_type: MessageType,
        message_string: String,
    ) -> Result<(), TunnelError> {
        self.control_message_handler_tx
            .send(Message::new(message_type, message_string))
            .await?;
        Ok(())
    }
}

pub async fn tunnel_control_message_sender(
    flags: Flags,
    mut control_rx: mpsc::Receiver<Message>,
    tunnel_client: Arc<TunnelClient>,
) {
    let mut stream_tx = tunnel_client.stream_tx.lock().await;
    loop {
        select! {
            biased;
            _global_cancalled = flags.global_cancellation_token.cancelled() => { break; },
            _client_cancealled = flags.local_cancellation_token.cancelled() => { break; },
            received_request = control_rx.recv() => {
                let Some(message) = received_request else {
                    flags.local_cancellation_token.cancel();
                    break;
                };

                select! {
                    biased;
                    _global_cancalled = flags.global_cancellation_token.cancelled() => { break; },
                    _client_cancealled = flags.local_cancellation_token.cancelled() => { break; },
                    write_result = send_message(&mut stream_tx, &message) => {
                        if let Err(error) = write_result {
                            log(
                                Level::Warning,
                                format!(
                                    "Unable to send message to client {}: {:?}",
                                    tunnel_client.addr.to_string(),
                                    error
                                )
                                .as_str(),
                                "core::tunnel::message_handler::tunnel_control_message_sender"
                            )
                            .await;
                            flags.local_cancellation_token.cancel();
                            break;
                        }
                    }
                }
            }
        }
    }
}
