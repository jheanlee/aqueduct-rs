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

pub struct Migration {
    pub name: &'static str,
    pub sql: &'static str,
}

macro_rules! include_up_migration {
    ($migration: literal) => {
        Migration {
            name: $migration,
            sql: include_str!(concat!(
                "../../../migration/src/migrations_raw_sql/up/",
                $migration,
                ".sql"
            )),
        }
    };
}

pub const BOOTSTRAP_SQL: &str =
    include_str!("../../../migration/src/migrations_raw_sql/bootstrap.sql");
pub const MIGRATION_001: Migration = include_up_migration!("Migration20260806140916");
pub const MIGRATION_002: Migration = include_up_migration!("Migration20260815064212");
