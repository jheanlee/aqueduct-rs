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
use crate::api::tunnel::users::{ModifyTunnelUserPasswordBody, NewTunnelUserBody};
use crate::common::auth_manager::AuthManager;
use crate::orm::error::Error;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, NaiveDateTime, Utc};
use entity::entities::tunnel_users::{ActiveModel, Column, Entity, ListUserPartialModel, Model};
use nanoid::nanoid;
use rand::{RngExt, rng};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
    Set,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub async fn authenticate_tunnel_user(
    db_connection: DatabaseConnection,
    auth_manager: Arc<AuthManager>,
    username: &str,
    password: &str,
) -> Result<(String, bool), Error> {
    let user = Entity::find()
        .filter(Column::Username.eq(username.to_lowercase()))
        .one(&db_connection)
        .await?
        .ok_or(Error::Unauthorized)?;
    if auth_manager
        .verify_password(password.to_string(), user.hashed_password.clone())
        .await?
    {
        let mut user_active_model = user.into_active_model();
        user_active_model.last_login = Set(Utc::now().naive_utc());
        let updated = user_active_model.update(&db_connection).await?;
        Ok((updated.id, updated.administrator))
    } else {
        Err(Error::Unauthorized)
    }
}

pub async fn authenticate_tunnel_token(
    db_connection: DatabaseConnection,
    token: &str,
) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    hasher.update(token);
    let hashed_token = BASE64_STANDARD.encode(hasher.finalize().to_vec());

    let mut user = Entity::find()
        .filter(Column::Token.eq(hashed_token))
        .one(&db_connection)
        .await?
        .ok_or(Error::Unauthorized)?
        .into_active_model();
    user.last_login = Set(Utc::now().naive_utc());
    Ok(user.update(&db_connection).await?.id)
}

pub async fn new_tunnel_user(
    db_connection: DatabaseConnection,
    auth_manager: Arc<AuthManager>,
    mut new_user: NewTunnelUserBody,
) -> Result<(), Error> {
    new_user.username = new_user.username.to_lowercase();
    if get_user_by_username(db_connection.clone(), new_user.username.as_str())
        .await
        .is_ok()
    {
        Err(Error::Conflict)?
    }

    let hashed_password = auth_manager.hash_password(new_user.password).await?;
    let mut token_bytes = [0u8; 32];
    rng().fill(&mut token_bytes);
    let token = format!("aq_{}", bs58::encode(&token_bytes).into_string());

    let mut hasher = Sha256::new();
    hasher.update(token.as_str());
    let hashed_token = BASE64_STANDARD.encode(hasher.finalize().to_vec());

    let user = ActiveModel {
        id: Set(nanoid!()),
        username: Set(new_user.username),
        token: Set(hashed_token),
        hashed_password: Set(hashed_password),
        label: Set(new_user.label),
        last_login: Set(NaiveDateTime::from(DateTime::UNIX_EPOCH.naive_utc())),
        administrator: Set(new_user.administrator),
    };

    user.insert(&db_connection).await?;
    Ok(())
}

pub async fn modify_tunnel_user(
    db_connection: DatabaseConnection,
    auth_manager: Arc<AuthManager>,
    id: &str,
    modify_user: ModifyTunnelUserPasswordBody,
) -> Result<(), Error> {
    let mut user = Entity::find_by_id(id)
        .one(&db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();

    if let Some(password) = modify_user.password {
        let hashed_password = auth_manager.hash_password(password).await?;
        user.hashed_password = Set(hashed_password);
    }

    user.label = Set(modify_user.label);
    user.administrator = Set(modify_user.administrator);

    user.update(&db_connection).await?;
    Ok(())
}

pub async fn rotate_token(db_connection: DatabaseConnection, id: &str) -> Result<String, Error> {
    let mut user = Entity::find_by_id(id)
        .one(&db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();

    let mut token_bytes = [0u8; 32];
    rng().fill(&mut token_bytes);
    let token = format!("aq_{}", bs58::encode(&token_bytes).into_string());

    let mut hasher = Sha256::new();
    hasher.update(token.as_str());
    let hashed_token = BASE64_STANDARD.encode(hasher.finalize().to_vec());

    user.token = Set(hashed_token);

    user.update(&db_connection).await?;
    Ok(token)
}

pub async fn delete_tunnel_user(db_connection: DatabaseConnection, id: &str) -> Result<(), Error> {
    let user = Entity::find_by_id(id)
        .one(&db_connection)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();
    user.delete(&db_connection).await?;
    Ok(())
}

async fn get_user_by_username(
    db_connection: DatabaseConnection,
    username: &str,
) -> Result<Model, Error> {
    Ok(Entity::find()
        .filter(Column::Username.eq(username.to_lowercase()))
        .one(&db_connection)
        .await?
        .ok_or(Error::NotFound)?)
}

pub async fn list_users(
    db_connection: DatabaseConnection,
) -> Result<Vec<ListUserPartialModel>, Error> {
    Ok(Entity::find()
        .into_partial_model()
        .all(&db_connection)
        .await?)
}
