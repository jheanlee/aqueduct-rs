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

#[derive(Debug)]
pub enum DbError {
    NotFound,
    DatabaseErr(sea_orm::DbErr),
    CommonError(crate::common::error::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Resource not found"),
            Self::DatabaseErr(error) => write!(f, "DbErr: {error}"),
            Self::CommonError(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<sea_orm::DbErr> for DbError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::DatabaseErr(error)
    }
}

impl From<crate::common::error::Error> for DbError {
    fn from(error: crate::common::error::Error) -> Self {
        Self::CommonError(error)
    }
}
