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

use crate::core::message::v1::common::MessageTypeV1;
use tokio_util::bytes::Bytes;

#[derive(Clone)]
pub enum MessageType {
    Heartbeat,
    Service, //  service connection
    Proxy,   //  proxy connection
    Close,
    Empty, //  placeholder
    Error,
}

#[derive(Clone)]
pub struct Message {
    pub message_version: u8,
    pub message_type: MessageType,
    pub message_payload: Bytes,
}

impl Message {
    pub fn new(message_version: u8, message_type: MessageType, message_payload: &str) -> Self {
        Self {
            message_version,
            message_type,
            message_payload: Bytes::copy_from_slice(message_payload.as_bytes()),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ServiceMessage {
    pub auth: ServiceAuth,
}

#[derive(serde::Deserialize)]
pub enum ServiceAuth {
    Token { token: String },
    Password { username: String, password: String },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ClientServiceMessage {
    pub port: u16,
    pub secret: String,
}

#[derive(serde::Deserialize)]
pub struct ProxyMessage {
    pub proxy_id: String,
}

impl From<MessageTypeV1> for MessageType {
    fn from(value: MessageTypeV1) -> Self {
        match value {
            MessageTypeV1::Heartbeat => MessageType::Heartbeat,
            MessageTypeV1::Service => MessageType::Service,
            MessageTypeV1::Proxy => MessageType::Proxy,
            MessageTypeV1::Close => MessageType::Close,
            MessageTypeV1::Empty => MessageType::Empty,
            MessageTypeV1::Error => MessageType::Error,
        }
    }
}
