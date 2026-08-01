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
use crate::orm::usage_data::{TimestampBucketSize, get_tunnel_usage_data};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http;
use axum::response::Response;
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, to_string};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct GetTunnelUsageQuery {
    pub resolution: TimestampBucketSize,
    pub query_start: NaiveDateTime,
    pub query_end: NaiveDateTime,
}

pub async fn get_tunnel_usage_overall(
    State(api_state): State<Arc<ApiState>>,
    Query(body): Query<GetTunnelUsageQuery>,
) -> Result<Response<Body>, Error> {
    let usage_points = get_tunnel_usage_data(
        api_state.shared.db_connection.clone(),
        None,
        body.resolution,
        body.query_start,
        body.query_end,
    )
    .await?;
    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::from(to_string(&json!({
        "usage_data_points": usage_points
    }))?);
    Ok(response_builder.body(response_body)?)
}

#[derive(Deserialize)]
pub struct GetTunnelUsageByUserPath {
    pub id: String,
}
pub async fn get_tunnel_usage_by_user(
    State(api_state): State<Arc<ApiState>>,
    Path(path): Path<GetTunnelUsageByUserPath>,
    Query(body): Query<GetTunnelUsageQuery>,
) -> Result<Response<Body>, Error> {
    let usage_points = get_tunnel_usage_data(
        api_state.shared.db_connection.clone(),
        Some(path.id),
        body.resolution,
        body.query_start,
        body.query_end,
    )
    .await?;
    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::from(to_string(&json!({
        "usage_data_points": usage_points
    }))?);
    Ok(response_builder.body(response_body)?)
}
