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
use crate::migration::error::Error;
use crate::migration::error::Error::{
    FutureMigrationVersion, InvalidMigrationRecord, UnknownMigrationVersion,
};
use crate::orm::mikro_orm_migrations::get_latest_migration_record;
use sea_orm::DatabaseConnection;
use tracing::{error, warn};

pub struct Migration {
    name: &'static str,
    sql: &'static str,
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

const BOOTSTRAP_SQL: &str = include_str!("../../../migration/src/migrations_raw_sql/bootstrap.sql");
pub const MIGRATION_001: Migration = include_up_migration!("Migration20260806140916");

pub async fn migrate(db_connection: DatabaseConnection) -> Result<(), Error> {
    const LATEST_MIGRATION_VERSION: &str = MIGRATION_001.name;

    let db_connection_pool = db_connection.get_postgres_connection_pool();

    if let Err(error) = sqlx::raw_sql(BOOTSTRAP_SQL)
        .execute(db_connection_pool)
        .await
    {
        error!("Failed to execute migration bootstrap: {}", error);
        Err(error)?;
    };

    let latest_migration_record = get_latest_migration_record(&db_connection).await?;

    if latest_migration_record
        .as_ref()
        .and_then(|v| v.name.as_deref())
        == Some(LATEST_MIGRATION_VERSION)
    {
        warn!("Database up to date. No changes applied");
        return Ok(());
    }

    let current_migration_version = match latest_migration_record {
        None => 0,
        Some(record) => match record.name {
            None => Err(InvalidMigrationRecord)?,
            Some(name) if name == MIGRATION_001.name => 1,
            Some(name) if name.as_str() < LATEST_MIGRATION_VERSION => Err(UnknownMigrationVersion)?,
            Some(name) if name.as_str() > LATEST_MIGRATION_VERSION => Err(FutureMigrationVersion)?,
            _ => Err(UnknownMigrationVersion)?,
        },
    };

    if current_migration_version == 0 {
        if let Err(error) = sqlx::raw_sql(MIGRATION_001.sql)
            .execute(db_connection_pool)
            .await
        {
            error!(
                "Failed to execute migration 001 ('{}'): {}",
                MIGRATION_001.name, error
            );
            Err(error)?;
        }
    }

    warn!("Migrated up to '{}'", LATEST_MIGRATION_VERSION);
    Ok(())
}
