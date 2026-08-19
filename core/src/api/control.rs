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
use crate::api::jwt::auth::{access_token_middleware, login, logout, refresh_token};
use crate::api::jwt::key::JwtKeyPair;
use crate::api::status::system::get_system_status;
use crate::api::status::tunnel::{get_tunnel_status, get_tunnel_status_overall};
use crate::api::tunnel::settings::{get_settings, set_settings};
use crate::api::tunnel::usage::{get_tunnel_usage_by_user, get_tunnel_usage_overall};
use crate::api::tunnel::users::{
    delete_tunnel_user, list_tunnel_users, modify_tunnel_user_password, new_tunnel_user,
    rotate_token,
};
use crate::common::model::Shared;
use crate::common::tunnel_info::TunnelInfo;
use crate::config::access_handler::update_access_ip_tables;
use crate::system_info::collector::SystemInfo;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};
use dashmap::DashMap;
use ip_network_table::IpNetworkTable;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::RwLock;
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

pub async fn api_control(
    api_host: SocketAddr,
    api_state: Arc<ApiState>,
    cancellation_token: CancellationToken,
) {
    let api_state_clone = api_state.clone();
    let cancellation_token_clone = cancellation_token.clone();
    tokio::spawn(async move {
        loop {
            select! {
                biased;
                _ = cancellation_token_clone.cancelled() => {
                    break;
                }
                _ = sleep(Duration::from_secs(300)) => {
                    let deadline = Instant::now();
                    api_state_clone.jti_map.retain(|_, value| value.2 > deadline);
                }
            }
        }
    });

    let with_access_update_router = Router::new()
        .route("/api/tunnel/settings", put(set_settings))
        .layer(middleware::from_fn_with_state(
            api_state.clone(),
            update_access_middleware,
        ));

    #[cfg(feature = "web-ui")]
    let web_ui_router = {
        use crate::api::static_files::handlers::{index_handler, static_file_handler};
        Router::new()
            .route("/{*path}", get(static_file_handler))
            .route("/", get(index_handler))
    };

    let api = Router::new()
        .route("/api/tunnel/users", post(new_tunnel_user))
        .route("/api/tunnel/users", get(list_tunnel_users))
        .route("/api/tunnel/users/{id}", put(modify_tunnel_user_password))
        .route("/api/tunnel/users/{id}", delete(delete_tunnel_user))
        .route("/api/tunnel/usage", get(get_tunnel_usage_overall))
        .route("/api/tunnel/usage/{id}", get(get_tunnel_usage_by_user))
        .route("/api/tunnel/users/{id}/token/rotate", post(rotate_token))
        .merge(with_access_update_router)
        .route("/api/tunnel/settings", get(get_settings))
        .route("/api/status/tunnel", get(get_tunnel_status_overall))
        .route("/api/status/realtime/system", get(get_system_status))
        .route("/api/status/realtime/tunnel", get(get_tunnel_status))
        .layer(middleware::from_fn_with_state(
            api_state.clone(),
            access_token_middleware,
        ))
        .route("/api/refresh", post(refresh_token))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .with_state(api_state);

    #[cfg(feature = "web-ui")]
    let api = api.merge(web_ui_router);

    match TcpListener::bind(api_host).await {
        Ok(api_listener) => {
            info!(
                "API service listening on {}:{}",
                api_host.ip(),
                api_host.port()
            );

            select! {
                biased;
                _ = cancellation_token.cancelled() => {}
                serve_result = axum::serve(api_listener, api) => {
                    if let Err(error) = serve_result {
                        error!("Failed to start api services: {:?}", error);
                    }
                }
            }
        }
        Err(error) => {
            error!("Failed to start api services: {:?}", error);
        }
    };
}

async fn update_access_middleware(
    State(api_state): State<Arc<ApiState>>,
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    if let Err(error) = update_access_ip_tables(
        &api_state.shared.db_connection,
        api_state.whitelist_table.clone(),
        api_state.blacklist_table.clone(),
    )
    .await
    {
        warn!("Unable to update the access tables: {:?}", error);
    }
    response
}

pub struct ApiState {
    pub start_time: Instant,
    pub shared: Shared,
    pub system_info: Arc<SystemInfo>,
    pub tunnel_info: Arc<TunnelInfo>,
    pub whitelist_table: Arc<RwLock<IpNetworkTable<()>>>,
    pub blacklist_table: Arc<RwLock<IpNetworkTable<()>>>,
    pub jti_map: DashMap<String, (String, String, Instant)>,
    pub refresh_token_keys: JwtKeyPair,
    pub access_token_keys: JwtKeyPair,
}
