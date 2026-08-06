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
use crate::core::message::common::{MESSAGE_VERSION_BYTES, MessageBuilderVersioned};
use crate::core::message::error::MessageBuildError;
use crate::core::message::message::Message;
use crate::core::message::v1::common::{
    MESSAGE_PAYLOAD_MAX_LEN_V1, MESSAGE_TYPE_BYTES_V1, MessageTypeV1,
};
use tokio_util::bytes::{BufMut, BytesMut};

pub struct MessageBuilderV1;

impl MessageBuilderVersioned for MessageBuilderV1 {
    fn encode(message: &Message, buffer: &mut BytesMut) -> Result<(), MessageBuildError> {
        if message.message_payload.len() > MESSAGE_PAYLOAD_MAX_LEN_V1 {
            return Err(MessageBuildError::InvalidStringLength);
        }

        buffer
            .reserve(MESSAGE_VERSION_BYTES + MESSAGE_TYPE_BYTES_V1 + message.message_payload.len());

        buffer.put_u8(message.message_version);
        buffer.put_u8(MessageTypeV1::from(message.message_type.clone()).as_u8());
        buffer.extend_from_slice(&message.message_payload);
        Ok(())
    }
}
