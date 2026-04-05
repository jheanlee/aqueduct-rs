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

#[derive(Debug, Copy, Clone)]
pub enum MessageError {
    MessageEmpty,
    MessageTooLong,
    InvalidType,
    InvalidString,
}

impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageError::MessageEmpty => write!(f, "message cannot be empty"),
            MessageError::MessageTooLong => write!(f, "message length exceeded limit"),
            MessageError::InvalidType => write!(f, "invalid message type"),
            MessageError::InvalidString => write!(f, "invalid message string"),
        }
    }
}

impl std::error::Error for MessageError {}
