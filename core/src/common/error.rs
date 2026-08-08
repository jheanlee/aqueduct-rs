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
use crate::core::tunnel::error::TunnelError;

#[derive(Debug)]
pub enum Error {
    MessageBuild(MessageBuildError),
    MessageParse(MessageParseError),
    Tunnel(TunnelError),
    Acquire(tokio::sync::AcquireError),
    Argon2Hash(argon2::password_hash::Error),
    TokioJoin(tokio::task::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MessageBuild(e) => write!(f, "MessageBuildError: {e}"),
            Error::MessageParse(e) => write!(f, "MessageParseError: {e}"),
            Error::Tunnel(e) => write!(f, "TunnelError: {e}"),
            Error::Acquire(e) => write!(f, "AcquireError: {e}"),
            Error::Argon2Hash(e) => write!(f, "Argon2HashError: {e}"),
            Error::TokioJoin(e) => write!(f, "TokioJoinError: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<TunnelError> for Error {
    fn from(error: TunnelError) -> Self {
        Self::Tunnel(error)
    }
}

impl From<MessageBuildError> for Error {
    fn from(error: MessageBuildError) -> Self {
        Self::MessageBuild(error)
    }
}

impl From<MessageParseError> for Error {
    fn from(error: MessageParseError) -> Self {
        Self::MessageParse(error)
    }
}

impl From<tokio::sync::AcquireError> for Error {
    fn from(error: tokio::sync::AcquireError) -> Self {
        Self::Acquire(error)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Argon2Hash(error)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::TokioJoin(error)
    }
}
