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
use axum::body::Body;
use axum::extract::State;
use axum::http;
use axum::response::Response;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub async fn get_system_status(
    State(api_state): State<Arc<ApiState>>,
) -> Result<Response<Body>, Error> {
    let body = json!({
        "cpu_usage": f32::from_bits(api_state.system_info.cpu_usage.load(Ordering::Relaxed)),
        "used_memory": api_state.system_info.used_memory,
        "total_memory": api_state.system_info.total_memory,
        "process_cpu_usage": f32::from_bits(api_state.system_info.process_cpu_usage.load(Ordering::Relaxed)),
        "process_memory": api_state.system_info.process_memory,
        "process_fd_count": api_state.system_info.process_fd_count,
    });

    let response_builder =
        Response::builder().header(http::header::CONTENT_TYPE, "application/json");
    Ok(response_builder.body(Body::from(body.to_string()))?)
}
