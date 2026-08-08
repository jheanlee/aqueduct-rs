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
use crate::core::message::error::MessageParseError;
use crate::core::message::types::MessageType;

pub const MESSAGE_VERSION_V1: u8 = 1;
pub const MESSAGE_TYPE_BYTES_V1: usize = 1;
pub const MESSAGE_PAYLOAD_MAX_LEN_V1: usize = 254;

pub enum MessageTypeV1 {
    Heartbeat,
    Service, //  service connection
    Proxy,   //  proxy connection
    Close,
    Empty, //  placeholder
    Error,
}

impl MessageTypeV1 {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Heartbeat => 0x10,
            Self::Service => 0x11,
            Self::Proxy => 0x12,
            Self::Close => 0xf0,
            Self::Empty => 0xfe,
            Self::Error => 0xff,
        }
    }

    pub fn from_u8(message_type: u8) -> Result<Self, MessageParseError> {
        match message_type {
            0x10 => Ok(Self::Heartbeat),
            0x11 => Ok(Self::Service),
            0x12 => Ok(Self::Proxy),
            0xf0 => Ok(Self::Close),
            0xfe => Ok(Self::Empty),
            0xff => Ok(Self::Error),
            _ => Err(MessageParseError::Type),
        }
    }
}

impl From<MessageType> for MessageTypeV1 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::Heartbeat => Self::Heartbeat,
            MessageType::Service => Self::Service,
            MessageType::Proxy => Self::Proxy,
            MessageType::Close => Self::Close,
            MessageType::Empty => Self::Empty,
            MessageType::Error => Self::Error,
        }
    }
}
