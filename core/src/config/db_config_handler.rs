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
use crate::config::error::ConfigError;
use crate::orm::settings::read_settings;
use sea_orm::DbConn;

pub struct DbConfig {
    pub whitelist: bool,
    pub blacklist: bool,
}

pub async fn read_db_config(db_connection: DbConn) -> Result<DbConfig, ConfigError> {
    let mut db_config = DbConfig {
        whitelist: false,
        blacklist: true,
    };
    let settings = read_settings(db_connection).await?;
    for entry in settings {
        match entry.key.as_str() {
            "whitelist" => {
                db_config.whitelist = entry.value.parse().map_err(|_| {
                    ConfigError::ParseError((
                        "database settings".to_string(),
                        "whitelist".to_string(),
                    ))
                })?;
            }
            "blacklist" => {
                db_config.blacklist = entry.value.parse().map_err(|_| {
                    ConfigError::ParseError((
                        "database settings".to_string(),
                        "blacklist".to_string(),
                    ))
                })?;
            }
            _ => {}
        }
    }
    Ok(db_config)
}
