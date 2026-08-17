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
use crate::orm::error::Error;
use entity::entities::mikro_orm_migrations::{Column, Entity, Model};
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

pub async fn get_latest_migration_record(
    db_connection: &DatabaseConnection,
) -> Result<Option<Model>, Error> {
    Ok(Entity::find()
        .order_by_desc(Column::Name)
        .one(db_connection)
        .await?)
}

pub fn migration_version(record: Option<Model>) -> i32 {
    match record {
        None => 0,
        Some(record) => match record.name {
            None => -1, //  InvalidMigrationRecord
            Some(name) if name == MIGRATION_001.name => 1,
            Some(name) if name == MIGRATION_002.name => 2,
            Some(name) if name.as_str() < LATEST_MIGRATION_VERSION => -2, //  UnknownMigrationVersion
            Some(name) if name.as_str() > LATEST_MIGRATION_VERSION => -3, //  FutureMigrationVersion
            _ => -2, //  UnknownMigrationVersion
        },
    }
}

pub struct Migration {
    pub name: &'static str,
    #[cfg(feature = "migration")]
    pub sql: &'static str,
}

macro_rules! include_up_migration {
    ($migration: literal) => {
        Migration {
            name: $migration,
            #[cfg(feature = "migration")]
            sql: include_str!(concat!(
                "../../../migration/src/migrations_raw_sql/up/",
                $migration,
                ".sql"
            )),
        }
    };
}

pub const MIGRATION_001: Migration = include_up_migration!("Migration20260806140916");
pub const MIGRATION_002: Migration = include_up_migration!("Migration20260815064212");
pub const LATEST_MIGRATION_VERSION: &str = MIGRATION_002.name;
