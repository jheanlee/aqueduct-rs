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

use crate::core::message::error::MessageError;
use crate::core::message::message::Message;
use futures::sink::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;
use tokio_util::bytes::Bytes;
use tokio_util::codec::LengthDelimitedCodec;

#[derive(Debug)]
pub enum Error {
    MessageError(MessageError),
    IoError(std::io::Error),
    ClientClosed,
}

impl From<MessageError> for Error {
    fn from(value: MessageError) -> Self {
        Self::MessageError(value)
    }
}

pub async fn read_message(stream: &mut (impl AsyncReadExt + Unpin)) -> Result<Message, Error> {
    let mut reader = LengthDelimitedCodec::builder()
        .length_field_offset(0)
        .length_field_type::<u8>()
        .length_adjustment(0)
        .new_read(stream);
    let read_result = reader.next().await;

    match read_result {
        Some(Ok(bytes_read)) => Ok(Message::from_bytes(bytes_read.as_ref(), bytes_read.len())?),
        Some(Err(error)) => Err(Error::IoError(error)),
        None => Err(Error::ClientClosed),
    }
}

pub async fn send_message(
    stream: &mut (impl AsyncWriteExt + Unpin),
    message: &Message,
) -> Result<usize, Error> {
    let mut writer = LengthDelimitedCodec::builder()
        .length_field_offset(0)
        .length_field_type::<u8>()
        .length_adjustment(0)
        .new_write(stream);
    let message_bytes = message.to_vec()?;
    let nbytes = message_bytes.len();

    match writer.send(Bytes::from(message_bytes)).await {
        Ok(_) => Ok(nbytes),
        Err(error) => Err(Error::IoError(error)),
    }
}
