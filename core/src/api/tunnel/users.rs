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
use crate::orm::tunnel_user::{ModifyTunnelUserPasswordBody, NewTunnelUserBody, list_users};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, http};
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use std::sync::Arc;

#[derive(Serialize)]
pub struct ListUserPartialModelSafe {
    pub id: String,
    pub username: String,
    pub label: Vec<String>,
    pub last_login: i64,
    pub administrator: bool,
}
pub async fn list_tunnel_users(
    State(api_state): State<Arc<ApiState>>,
) -> Result<Response<Body>, Error> {
    let users: Vec<ListUserPartialModelSafe> = list_users(api_state.shared.db_connection.clone())
        .await?
        .into_iter()
        .map(|v| ListUserPartialModelSafe {
            id: v.id,
            username: v.username,
            label: v.label,
            last_login: v.last_login.and_utc().timestamp(),
            administrator: v.administrator,
        })
        .collect();
    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    Ok(response_builder.body(Body::from(to_string(&users)?))?)
}

pub async fn new_tunnel_user(
    State(api_state): State<Arc<ApiState>>,
    Json(body): Json<NewTunnelUserBody>,
) -> Result<impl IntoResponse, Error> {
    crate::orm::tunnel_user::new_tunnel_user(
        api_state.shared.db_connection.clone(),
        api_state.shared.auth_manager.clone(),
        body,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ModifyTunnelUserPasswordPath {
    pub id: String,
}
pub async fn modify_tunnel_user_password(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<ModifyTunnelUserPasswordPath>,
    Json(body): Json<ModifyTunnelUserPasswordBody>,
) -> Result<impl IntoResponse, Error> {
    crate::orm::tunnel_user::modify_tunnel_user(
        api_state.shared.db_connection.clone(),
        api_state.shared.auth_manager.clone(),
        path.id.as_str(),
        body,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct DeleteTunnelUserPath {
    pub id: String,
}
pub async fn delete_tunnel_user(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<DeleteTunnelUserPath>,
) -> Result<impl IntoResponse, Error> {
    crate::orm::tunnel_user::delete_tunnel_user(
        api_state.shared.db_connection.clone(),
        path.id.as_str(),
    )
    .await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct RotateTokenPath {
    pub id: String,
}
pub async fn rotate_token(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<RotateTokenPath>,
) -> Result<Response<Body>, Error> {
    let new_token = crate::orm::tunnel_user::rotate_token(
        api_state.shared.db_connection.clone(),
        path.id.as_str(),
    )
    .await?;
    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::from(
        json!({
            "token": new_token
        })
        .to_string(),
    );

    Ok(response_builder.body(response_body)?)
}
