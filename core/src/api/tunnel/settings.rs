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
use crate::config::db_config_handler::read_db_config;
use crate::orm::blacklist::{get_blacklist, set_blacklist};
use crate::orm::settings::{SettingsKey, read_settings, set_settings_value};
use crate::orm::whitelist::{get_whitelist, set_whitelist};
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, http};
use ip_network::IpNetwork;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::sync::Arc;
use tracing::warn;

#[derive(Serialize)]
pub struct GetSettingsResponse {
    pub blacklist_enabled: bool,
    pub blacklist: Vec<IpNetwork>,
    pub whitelist_enabled: bool,
    pub whitelist: Vec<IpNetwork>,
}

pub async fn get_settings(State(api_state): State<Arc<ApiState>>) -> Result<Response<Body>, Error> {
    let settings = read_db_config(api_state.shared.db_connection.clone()).await?;
    let blacklist = get_blacklist(api_state.shared.db_connection.clone()).await?;
    let whitelist = get_whitelist(api_state.shared.db_connection.clone()).await?;

    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::from(to_string(&GetSettingsResponse {
        blacklist_enabled: settings.blacklist,
        blacklist,
        whitelist_enabled: settings.whitelist,
        whitelist,
    })?);

    Ok(response_builder.body(response_body)?)
}

#[derive(Deserialize, Serialize)]
pub struct SetSettingsBody {
    pub blacklist_enabled: bool,
    pub blacklist: Vec<String>,
    pub whitelist_enabled: bool,
    pub whitelist: Vec<String>,
}
pub async fn set_settings(
    State(api_state): State<Arc<ApiState>>,
    Json(body): Json<SetSettingsBody>,
) -> Result<impl IntoResponse, Error> {
    let whitelist = body
        .whitelist
        .iter()
        .map(|entry| {
            entry.parse().map_err(|e| {
                warn!("{:?}", e);
                Error::DatabaseError(crate::orm::error::Error::BadRequest)
            })
        })
        .collect::<Result<Vec<IpNetwork>, Error>>()?;
    let blacklist = body
        .blacklist
        .iter()
        .map(|entry| {
            entry
                .parse()
                .map_err(|_| Error::DatabaseError(crate::orm::error::Error::BadRequest))
        })
        .collect::<Result<Vec<IpNetwork>, Error>>()?;

    set_settings_value(
        api_state.shared.db_connection.clone(),
        SettingsKey::Blacklist,
        body.blacklist_enabled.to_string(),
    )
    .await?;
    set_settings_value(
        api_state.shared.db_connection.clone(),
        SettingsKey::Whitelist,
        body.whitelist_enabled.to_string(),
    )
    .await?;

    set_blacklist(api_state.shared.db_connection.clone(), blacklist).await?;
    set_whitelist(api_state.shared.db_connection.clone(), whitelist).await?;

    Ok(StatusCode::OK)
}
