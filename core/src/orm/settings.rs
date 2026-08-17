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
use entity::entities::settings::{ActiveModel, Column, Entity, Model};
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use strum::AsRefStr;

#[derive(AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum SettingsKey {
    Whitelist,
    Blacklist,
}

pub async fn read_settings(db_connection: &DatabaseConnection) -> Result<Vec<Model>, Error> {
    Ok(Entity::find().all(db_connection).await?)
}

pub async fn set_settings_value(
    db_connection: &DatabaseConnection,
    key: SettingsKey,
    value: String,
) -> Result<(), Error> {
    match key {
        SettingsKey::Whitelist => {
            value.parse::<bool>().map_err(|_| Error::BadRequest)?;
        }
        SettingsKey::Blacklist => {
            value.parse::<bool>().map_err(|_| Error::BadRequest)?;
        }
    }

    upsert_settings_entry(db_connection, key.as_ref().to_string(), value).await
}

async fn upsert_settings_entry(
    db_connection: &DatabaseConnection,
    key: String,
    value: String,
) -> Result<(), Error> {
    let model = ActiveModel {
        key: Set(key),
        value: Set(value),
    };
    Entity::insert(model)
        .on_conflict(
            OnConflict::column(Column::Key)
                .update_column(Column::Value.to_owned())
                .clone(),
        )
        .exec(db_connection)
        .await?;
    Ok(())
}
