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
use crate::common::auth_manager::AuthManager;
use crate::init::error::Error;
use crate::orm::blacklist::set_blacklist;
use crate::orm::mikro_orm_migrations::{get_latest_migration_record, migration_version};
use crate::orm::settings::{SettingsKey, set_settings_value};
use crate::orm::tunnel_user::{NewTunnelUserBody, new_tunnel_user};
use crate::orm::whitelist::set_whitelist;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{error, warn};

pub async fn initialize_database(db_connection: DatabaseConnection) -> Result<(), Error> {
    const MINIMAL_SUPPORTED_VERSION: i32 = 2;

    let migration_record = match get_latest_migration_record(&db_connection).await {
        Ok(value) => value,
        Err(crate::orm::error::Error::Database(sea_orm::DbErr::Query(
            sea_orm::RuntimeErr::SqlxError(error),
        ))) => match error.as_ref() {
            sea_orm::sqlx::Error::Database(db_err) => match db_err.code() {
                Some(code) if code == "42P01" => None,
                _ => {
                    let error = Err(crate::orm::error::Error::Database(sea_orm::DbErr::Query(
                        sea_orm::RuntimeErr::SqlxError(error),
                    )));
                    error!("{:?}", error);
                    error?
                }
            },
            _ => {
                let error = Err(crate::orm::error::Error::Database(sea_orm::DbErr::Query(
                    sea_orm::RuntimeErr::SqlxError(error),
                )));
                error!("{:?}", error);
                error?
            }
        },
        Err(error) => {
            error!("{error}");
            Err(error)?
        }
    };

    if let Err(error) = match migration_version(migration_record) {
        -1 => Err(Error::InvalidMigrationRecord),
        -2 => Err(Error::UnknownMigrationVersion),
        -3 => Err(Error::FutureMigrationVersion),
        0 => Err(Error::DatabaseNotInitialized),
        1..MINIMAL_SUPPORTED_VERSION => Err(Error::UnsupportedMigrationVersion),
        _ => Ok(()),
    } {
        error!("Failed to initialize database: {error}");
        Err(error)?
    }

    let auth_manager = Arc::new(AuthManager::new());

    //  user
    const DEFAULT_USER_NAME: &str = "admin";
    const DEFAULT_USER_PASSWORD: &str = "password";
    if let Err(error) = new_tunnel_user(
        db_connection.clone(),
        auth_manager,
        NewTunnelUserBody {
            username: DEFAULT_USER_NAME.to_string(),
            password: DEFAULT_USER_PASSWORD.to_string(),
            label: vec![],
            administrator: true,
        },
    )
    .await
    {
        error!("Failed to initialize database: {}", error);
        Err(error)?
    }

    //  settings
    if let Err(error) =
        set_settings_value(&db_connection, SettingsKey::Blacklist, "true".to_string()).await
    {
        error!("Failed to initialize database: {}", error);
        Err(error)?
    }

    if let Err(error) =
        set_settings_value(&db_connection, SettingsKey::Whitelist, "false".to_string()).await
    {
        error!("Failed to initialize database: {}", error);
        Err(error)?
    }

    if let Err(error) = set_blacklist(&db_connection, vec![]).await {
        error!("Failed to initialize database: {}", error);
        Err(error)?
    }

    if let Err(error) = set_whitelist(&db_connection, vec![]).await {
        error!("Failed to initialize database: {}", error);
        Err(error)?
    }

    warn!(
        r#"Initialized database:
        -------- User --------
        Username: {DEFAULT_USER_NAME}
        Password: {DEFAULT_USER_PASSWORD}
        -------- Settings --------
        blacklist: enabled
        blacklist ips: none
        whitelist: disabled
        whitelist ips: none"#
    );

    warn!("IMPORTANT: please change the default password using the management web UI");

    Ok(())
}
