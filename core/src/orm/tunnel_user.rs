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
use crate::common::model::Shared;
use crate::orm::error::Error;
use chrono::{DateTime, NaiveDateTime};
use entity::entities::tunnel_users::{ActiveModel, Column, Entity};
use nanoid::nanoid;
use rand::{RngExt, rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

pub async fn authenticate_tunnel_user(
    shared: Shared,
    username: &str,
    password: &str,
) -> Result<Option<String>, Error> {
    let user = Entity::find()
        .filter(Column::Username.eq(username))
        .one(&shared.db_connection)
        .await?
        .ok_or(Error::NotFound)?;
    if shared
        .auth_manager
        .verify_password(password.to_string(), user.hashed_password)
        .await?
    {
        Ok(Some(user.id))
    } else {
        Ok(None)
    }
}

pub async fn new_tunnel_user(
    shared: Shared,
    username: String,
    password: String,
) -> Result<(), Error> {
    //  TODO check user
    let hashed_password = shared.auth_manager.hash_password(password).await?;
    let mut token_bytes = [0u8; 32];
    rng().fill(&mut token_bytes);
    let token = format!("aq_{}", bs58::encode(&token_bytes).into_string());

    let user = ActiveModel {
        id: Set(nanoid!()),
        username: Set(username),
        token: Set(token),
        hashed_password: Set(hashed_password),
        label: Set(String::new()),
        last_login: Set(NaiveDateTime::from(DateTime::UNIX_EPOCH.naive_utc())),
    };

    user.insert(&shared.db_connection).await?;
    Ok(())
}

pub async fn modify_tunnel_user_password(
    shared: Shared,
    id: &str,
    new_password: String,
) -> Result<(), Error> {
    let mut user = Entity::find_by_id(id)
        .one(&shared.db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();

    let hashed_password = shared.auth_manager.hash_password(new_password).await?;
    user.hashed_password = Set(hashed_password);

    user.update(&shared.db_connection).await?;
    Ok(())
}

pub async fn rotate_token(shared: Shared, id: &str) -> Result<(), Error> {
    //  TODO api
    let mut user = Entity::find_by_id(id)
        .one(&shared.db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();

    let mut token_bytes = [0u8; 32];
    rng().fill(&mut token_bytes);
    let token = format!("aq_{}", bs58::encode(&token_bytes).into_string());

    user.token = Set(token);

    user.update(&shared.db_connection).await?;
    Ok(())
}

pub async fn delete_tunnel_user(shared: Shared, id: &str) -> Result<(), Error> {
    let user = Entity::find_by_id(id)
        .one(&shared.db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();
    user.delete(&shared.db_connection).await?;
    Ok(())
}
