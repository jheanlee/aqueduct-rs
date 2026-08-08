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
use crate::core::message::common::MessageParserVersioned;
use crate::core::message::error::MessageParseError;
use crate::core::message::types::Message;
use crate::core::message::v1::common::{MESSAGE_TYPE_BYTES_V1, MESSAGE_VERSION_V1, MessageTypeV1};
use tokio_util::bytes::Bytes;

pub struct MessageParserV1;

impl MessageParserVersioned for MessageParserV1 {
    fn parse(bytes: Bytes) -> Result<Message, MessageParseError> {
        if bytes.len() < MESSAGE_TYPE_BYTES_V1 {
            return Err(MessageParseError::Length);
        }

        let message_payload = bytes.slice(MESSAGE_TYPE_BYTES_V1..);

        //  string validation
        str::from_utf8(&message_payload).map_err(|_| MessageParseError::String)?;

        Ok(Message {
            message_version: MESSAGE_VERSION_V1,
            message_type: MessageTypeV1::from_u8(bytes[0])?.into(),
            message_payload: bytes.slice(MESSAGE_TYPE_BYTES_V1..),
        })
    }
}
