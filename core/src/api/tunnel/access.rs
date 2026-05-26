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
use crate::api::control::ApiState;
use crate::api::error::Error;
use crate::orm::blacklist::{BlacklistEntry, delete_blacklist};
use crate::orm::whitelist::{WhitelistEntry, delete_whitelist};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AddBlacklistBody {
    pub data: Vec<BlacklistEntry>,
}
pub async fn add_blacklist(
    State(api_state): State<Arc<ApiState>>,
    Json(body): Json<AddBlacklistBody>,
) -> Result<impl IntoResponse, Error> {
    crate::orm::blacklist::add_blacklist(api_state.shared.db_connection.clone(), body.data).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct RemoveBlacklistPath {
    pub id: i32,
}
pub async fn remove_blacklist(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<RemoveBlacklistPath>,
) -> Result<impl IntoResponse, Error> {
    delete_blacklist(api_state.shared.db_connection.clone(), path.id).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct AddWhitelistBody {
    pub data: Vec<WhitelistEntry>,
}
pub async fn add_whitelist(
    State(api_state): State<Arc<ApiState>>,
    Json(body): Json<AddWhitelistBody>,
) -> Result<impl IntoResponse, Error> {
    crate::orm::whitelist::add_whitelist(api_state.shared.db_connection.clone(), body.data).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct RemoveWhitelistPath {
    pub id: i32,
}
pub async fn remove_whitelist(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<RemoveWhitelistPath>,
) -> Result<impl IntoResponse, Error> {
    delete_whitelist(api_state.shared.db_connection.clone(), path.id).await?;
    Ok(StatusCode::OK)
}
