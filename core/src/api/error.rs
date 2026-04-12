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
use axum::response::{IntoResponse, Response};
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    DatabaseError(crate::orm::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "DatabaseError: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::DatabaseError(error) => error.into_response(),
        }
    }
}

impl From<crate::orm::error::Error> for Error {
    fn from(error: crate::orm::error::Error) -> Self {
        Self::DatabaseError(error)
    }
}
