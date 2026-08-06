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
use crate::core::message::error::{MessageBuildError, MessageParseError};
use crate::core::message::message::Message;
use crate::core::message::v1::builder::MessageBuilderV1;
use crate::core::message::v1::common::MESSAGE_VERSION_V1;
use crate::core::message::v1::parser::MessageParserV1;
use tokio_util::bytes::{Bytes, BytesMut};

pub const MESSAGE_VERSION_BYTES: usize = 1;

pub struct MessageParser;
impl MessageParser {
    pub fn parse(bytes: Bytes) -> Result<Message, MessageParseError> {
        if bytes.len() < MESSAGE_VERSION_BYTES {
            return Err(MessageParseError::InvalidLength);
        }

        match bytes[0] {
            MESSAGE_VERSION_V1 => MessageParserV1::parse(bytes.slice(1..)),
            _ => Err(MessageParseError::InvalidVersion),
        }
    }
}

pub struct MessageBuilder {}
impl MessageBuilder {
    pub fn encode(message: &Message, buffer: &mut BytesMut) -> Result<(), MessageBuildError> {
        match message.message_version {
            MESSAGE_VERSION_V1 => MessageBuilderV1::encode(message, buffer),
            _ => Err(MessageBuildError::InvalidVersion),
        }
    }
}

pub trait MessageParserVersioned {
    fn parse(bytes: Bytes) -> Result<Message, MessageParseError>;
}

pub trait MessageBuilderVersioned {
    fn encode(message: &Message, buffer: &mut BytesMut) -> Result<(), MessageBuildError>;
}
