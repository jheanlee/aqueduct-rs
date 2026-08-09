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

#[derive(Debug)]
pub enum Error {
    NotFound,
    Unauthorized,
    Conflict,
    BadRequest,
    Database(sea_orm::DbErr),
    Common(crate::common::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Resource not found"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::Conflict => write!(f, "Conflict"),
            Self::BadRequest => write!(f, "Bad request"),
            Self::Database(error) => write!(f, "DbErr: {error}"),
            Self::Common(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            Error::Conflict => StatusCode::CONFLICT.into_response(),
            Error::BadRequest => StatusCode::BAD_REQUEST.into_response(),
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Error::Common(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::Database(error)
    }
}

impl From<crate::common::error::Error> for Error {
    fn from(error: crate::common::error::Error) -> Self {
        Self::Common(error)
    }
}
