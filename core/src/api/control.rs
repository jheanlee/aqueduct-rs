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
use crate::api::tunnel::users::{delete_tunnel_user, modify_tunnel_user_password, new_tunnel_user};
use crate::common::log::{Level, log};
use crate::common::model::Shared;
use axum::Router;
use axum::routing::{delete, post, put};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::select;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct ApiState {
    pub shared: Shared,
}

pub async fn api_control(
    shared: Shared,
    cancellation_token: CancellationToken,
    api_host: SocketAddr,
) {
    let api = Router::new()
        .route("/api/tunnel/users", post(new_tunnel_user))
        .route("/api/tunnel/users/{id}", put(modify_tunnel_user_password))
        .route("/api/tunnel/users/{id}", delete(delete_tunnel_user))
        .with_state(ApiState { shared });

    match TcpListener::bind(api_host).await {
        Ok(api_listener) => {
            select! {
                biased;
                _ = cancellation_token.cancelled() => { return; }
                serve_result = axum::serve(api_listener, api) => {
                    if let Err(error) = serve_result {
                        log(
                            Level::Error,
                            format!("Failed to start api services: {:?}", error).as_str(),
                            "api::control",
                        )
                        .await;
                        return;
                    }
                }
            }
        }
        Err(error) => {
            log(
                Level::Error,
                format!("Failed to start api services: {:?}", error).as_str(),
                "api::control",
            )
            .await;
            return;
        }
    };
}
