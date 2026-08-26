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
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub enum Error {
    Database(crate::orm::error::Error),
    MigrationExec(sqlx::Error),
    InvalidMigrationRecord,
    FutureMigrationVersion,
    UnknownMigrationVersion,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(e) => write!(f, "DatabaseError: {e}"),
            Self::MigrationExec(e) => write!(f, "MigrationExecError: {e}"),
            Self::InvalidMigrationRecord => write!(f, "Invalid migration record in the database"),
            Self::FutureMigrationVersion => write!(
                f,
                "Record of (possibly) a future migration version found in the database. Please upgrade to a compatible version"
            ),
            Self::UnknownMigrationVersion => write!(
                f,
                "Record of a unknown migration version found in the database"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::MigrationExec(value)
    }
}

impl From<crate::orm::error::Error> for Error {
    fn from(value: crate::orm::error::Error) -> Self {
        Self::Database(value)
    }
}
