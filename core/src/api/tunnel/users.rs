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
use crate::orm;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewTunnelUserBody {
    pub username: String,
    pub password: String,
}
pub async fn new_tunnel_user(
    State(api_state): State<ApiState>,
    Json(body): Json<NewTunnelUserBody>,
) -> Result<impl IntoResponse, Error> {
    orm::tunnel_user::new_tunnel_user(api_state.shared, body.username, body.password).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ModifyTunnelUserPasswordPath {
    pub id: String,
}
#[derive(Deserialize)]
pub struct ModifyTunnelUserPasswordBody {
    pub new_password: String,
}
pub async fn modify_tunnel_user_password(
    State(api_state): State<ApiState>,
    Path(path): Path<ModifyTunnelUserPasswordPath>,
    Json(body): Json<ModifyTunnelUserPasswordBody>,
) -> Result<impl IntoResponse, Error> {
    orm::tunnel_user::modify_tunnel_user_password(
        api_state.shared,
        path.id.as_str(),
        body.new_password,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct DeleteTunnelUserPath {
    pub id: String,
}
pub async fn delete_tunnel_user(
    State(api_state): State<ApiState>,
    Path(path): Path<DeleteTunnelUserPath>,
) -> Result<impl IntoResponse, Error> {
    orm::tunnel_user::delete_tunnel_user(api_state.shared, path.id.as_str()).await?;
    Ok(StatusCode::OK)
}
