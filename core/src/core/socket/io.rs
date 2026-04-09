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
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

#[derive(Debug)]
pub enum Error {
    MessageError(MessageError),
    IoError(std::io::Error),
}

pub async fn read_message(
    stream: &mut ReadHalf<TlsStream<TcpStream>>,
    buffer: &mut [u8],
) -> Result<Message, Error> {
    let read_result = stream.read(buffer.as_mut()).await;

    match read_result {
        Ok(bytes_read) => {
            let message = Message::from_bytes(buffer, bytes_read);
            match message {
                Ok(message) => Ok(message),
                Err(error) => Err(Error::MessageError(error)),
            }
        }
        Err(error) => Err(Error::IoError(error)),
    }
}

pub async fn send_message(
    stream: &mut WriteHalf<TlsStream<TcpStream>>,
    message: &Message,
) -> Result<usize, Error> {
    let message_bytes = message.to_vec();

    match message_bytes {
        Ok(message_bytes) => match stream.write_all(message_bytes.as_slice()).await {
            Ok(_) => Ok(message_bytes.len()),
            Err(error) => Err(Error::IoError(error)),
        },
        Err(error) => Err(Error::MessageError(error)),
    }
}
