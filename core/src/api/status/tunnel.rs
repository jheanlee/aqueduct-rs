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
use crate::orm::tunnel_status::get_tunnel_status_data;
use crate::orm::usage_data::TimestampBucketSize;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http;
use axum::response::Response;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};
use std::sync::Arc;

pub async fn get_tunnel_status(
    State(api_state): State<Arc<ApiState>>,
) -> Result<Response<Body>, Error> {
    let body = json!({
        "uptime": api_state.start_time.elapsed().as_secs(),
        "active_service_count": api_state.tunnel_info.active_service_count,
        "active_external_connection_count": api_state.tunnel_info.active_external_connection_count,
    });
    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    Ok(response_builder.body(Body::from(body.to_string()))?)
}

#[derive(Deserialize)]
pub struct GetTunnelStatusQuery {
    pub resolution: TimestampBucketSize,
    pub query_start: NaiveDateTime,
    pub query_end: NaiveDateTime,
}

#[derive(Serialize)]
pub struct TunnelStatusModelSafe {
    pub bucket: i64,
    pub active_service_avg: i64,
    pub active_service_max: i64,
    pub external_connection_avg: i64,
    pub external_connection_max: i64,
}
pub async fn get_tunnel_status_overall(
    State(api_state): State<Arc<ApiState>>,
    Query(query): Query<GetTunnelStatusQuery>,
) -> Result<Response<Body>, Error> {
    let status_data: Vec<TunnelStatusModelSafe> = get_tunnel_status_data(
        api_state.shared.db_connection.clone(),
        query.resolution,
        query.query_start,
        query.query_end,
    )
    .await?
    .iter()
    .map(|v| TunnelStatusModelSafe {
        bucket: v.bucket.and_utc().timestamp(),
        active_service_avg: v.active_service_avg,
        active_service_max: v.active_service_max,
        external_connection_avg: v.external_connection_avg,
        external_connection_max: v.external_connection_max,
    })
    .collect();

    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    let response_body = Body::from(to_string(&json!({
        "status_data_points": status_data
    }))?);
    Ok(response_builder.body(response_body)?)
}
