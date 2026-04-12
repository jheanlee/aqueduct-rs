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
use crate::core::tunnel::error::TunnelError;

#[derive(Debug)]
pub enum Error {
    MessageError(MessageError),
    TunnelError(TunnelError),
    AcquireError(tokio::sync::AcquireError),
    OpensslError(openssl::error::ErrorStack),
    Argon2HashError(argon2::password_hash::Error),
    TokioJoinError(tokio::task::JoinError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MessageError(e) => write!(f, "MessageError: {e}"),
            Error::TunnelError(e) => write!(f, "TunnelError: {e}"),
            Error::AcquireError(e) => write!(f, "AcquireError: {e}"),
            Error::OpensslError(e) => write!(f, "OpensslError: {e}"),
            Error::Argon2HashError(e) => write!(f, "Argon2HashError: {e}"),
            Error::TokioJoinError(e) => write!(f, "TokioJoinError: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<TunnelError> for Error {
    fn from(error: TunnelError) -> Self {
        Self::TunnelError(error)
    }
}

impl From<MessageError> for Error {
    fn from(error: MessageError) -> Self {
        Self::MessageError(error)
    }
}

impl From<tokio::sync::AcquireError> for Error {
    fn from(error: tokio::sync::AcquireError) -> Self {
        Self::AcquireError(error)
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(error: openssl::error::ErrorStack) -> Self {
        Self::OpensslError(error)
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(error: argon2::password_hash::Error) -> Self {
        Self::Argon2HashError(error)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::TokioJoinError(error)
    }
}
