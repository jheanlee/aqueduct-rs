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
use crate::orm::mikro_orm_migrations::{
    LATEST_MIGRATION_VERSION, MIGRATION_001, MIGRATION_002, get_latest_migration_record,
    migration_version,
};
use sea_orm::DatabaseConnection;
use tracing::{error, warn};

pub const BOOTSTRAP_SQL: &str =
    include_str!("../../../migration/src/migrations_raw_sql/bootstrap.sql");

pub async fn migrate(db_connection: DatabaseConnection) -> Result<(), Error> {
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

    let mut current_migration_version = match migration_version(latest_migration_record) {
        -1 => {
            warn!("Invalid migration record in the database");
            Err(Error::InvalidMigrationRecord)?
        }
        -2 => {
            warn!("Record of a unknown migration version found in the database");
            Err(Error::UnknownMigrationVersion)?
        }
        -3 => {
            warn!(
                "Record of (possibly) a future migration version found in the database. Please upgrade to a compatible version"
            );
            Err(Error::FutureMigrationVersion)?
        }
        version => version,
    };

    if current_migration_version == 0
        && let Err(error) = sqlx::raw_sql(MIGRATION_001.sql)
            .execute(db_connection_pool)
            .await
            .inspect(|_| current_migration_version = 1)
    {
        error!(
            "Failed to execute migration 001 ('{}'): {}",
            MIGRATION_001.name, error
        );
        Err(error)?;
    }

    if current_migration_version == 1
        && let Err(error) = sqlx::raw_sql(MIGRATION_002.sql)
            .execute(db_connection_pool)
            .await
            .inspect(|_| current_migration_version = 2)
    {
        error!(
            "Failed to execute migration 002 ('{}'): {}",
            MIGRATION_002.name, error
        );
        Err(error)?;
    }

    warn!("Migrated up to '{}'", LATEST_MIGRATION_VERSION);
    Ok(())
}
