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
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt::Formatter;
use tracing::warn;

#[derive(Debug)]
pub enum Error {
    Database(crate::orm::error::Error),
    Json(serde_json::Error),
    Http(axum::http::Error),
    Jwt(jsonwebtoken::errors::Error),
    Io(std::io::Error),
    Config(crate::config::error::ConfigError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "DatabaseError: {e}"),
            Self::Json(e) => write!(f, "JsonError: {e}"),
            Self::Http(e) => write!(f, "HttpError: {e}"),
            Self::Jwt(e) => write!(f, "JWTError: {e}"),
            Self::Io(e) => write!(f, "IoError: {e}"),
            Self::Config(e) => write!(f, "ConfigError: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::Database(crate::orm::error::Error::Unauthorized) => {}
            Error::Database(crate::orm::error::Error::NotFound) => {}
            Error::Database(crate::orm::error::Error::BadRequest) => {}
            Error::Database(crate::orm::error::Error::Conflict) => {}
            ref error => {
                warn!("{:?}", error);
            }
        }

        match self {
            Error::Database(error) => error.into_response(),
            Error::Json(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Http(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Jwt(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Config(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<crate::orm::error::Error> for Error {
    fn from(error: crate::orm::error::Error) -> Self {
        Self::Database(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<axum::http::Error> for Error {
    fn from(error: axum::http::Error) -> Self {
        Self::Http(error)
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::Jwt(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<crate::config::error::ConfigError> for Error {
    fn from(error: crate::config::error::ConfigError) -> Self {
        match error {
            crate::config::error::ConfigError::OrmError(e) => Self::Database(e),
            e => Self::Config(e),
        }
    }
}
