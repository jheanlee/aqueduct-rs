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
use crate::core::message::types::Message;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum TunnelError {
    MpscMessageSendError(mpsc::error::SendError<Message>),
    MessageParseError(MessageParseError),
    IoError(std::io::Error),
    NoPortsAvailable,
    HmacInvalidLengthError(hmac::digest::InvalidLength),
    ClientClosed,
}

impl std::fmt::Display for TunnelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IoError: {e}"),
            Self::NoPortsAvailable => write!(f, "no ports available"),
            Self::MpscMessageSendError(e) => write!(f, "MpscSendError: {e}"),
            Self::HmacInvalidLengthError(e) => write!(f, "HmacError: {e}"),
            Self::ClientClosed => write!(f, "Client closed"),
            Self::MessageParseError(e) => write!(f, "MessageParseError: {e}"),
        }
    }
}

impl std::error::Error for TunnelError {}

impl From<std::io::Error> for TunnelError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<mpsc::error::SendError<Message>> for TunnelError {
    fn from(error: mpsc::error::SendError<Message>) -> Self {
        Self::MpscMessageSendError(error)
    }
}

impl From<hmac::digest::InvalidLength> for TunnelError {
    fn from(error: hmac::digest::InvalidLength) -> Self {
        Self::HmacInvalidLengthError(error)
    }
}

impl From<MessageParseError> for TunnelError {
    fn from(error: MessageParseError) -> Self {
        Self::MessageParseError(error)
    }
}
