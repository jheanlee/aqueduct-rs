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
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone)]
pub enum MessageParseError {
    Length,
    Version,
    Type,
    String,
}

#[derive(Debug, Copy, Clone)]
pub enum MessageBuildError {
    InvalidVersion,
    InvalidStringLength,
}

impl std::fmt::Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageParseError::Length => {
                write!(f, "invalid length")
            }
            MessageParseError::Version => {
                write!(f, "invalid version")
            }
            MessageParseError::Type => {
                write!(f, "invalid type")
            }
            MessageParseError::String => {
                write!(f, "invalid string")
            }
        }
    }
}

impl std::error::Error for MessageParseError {}

impl std::fmt::Display for MessageBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageBuildError::InvalidVersion => {
                write!(f, "invalid version")
            }
            MessageBuildError::InvalidStringLength => {
                write!(f, "invalid string")
            }
        }
    }
}

impl std::error::Error for MessageBuildError {}
