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
use crate::api::tunnel::access::{
    add_blacklist, add_whitelist, remove_blacklist, remove_whitelist,
};
use crate::api::tunnel::users::{
    delete_tunnel_user, get_tunnel_usage_by_user, modify_tunnel_user_password, new_tunnel_user,
    rotate_token,
};
use crate::common::model::Shared;
use crate::config::access_handler::update_access_ip_tables;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{delete, get, post, put};
use axum::{Router, middleware};
use ip_network_table::IpNetworkTable;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};

#[derive(Clone)]
pub struct ApiState {
    pub shared: Shared,
    pub whitelist_table: Arc<RwLock<IpNetworkTable<()>>>,
    pub blacklist_table: Arc<RwLock<IpNetworkTable<()>>>,
}

pub async fn api_control(
    api_host: SocketAddr,
    shared: Shared,
    whitelist_table: Arc<RwLock<IpNetworkTable<()>>>,
    blacklist_table: Arc<RwLock<IpNetworkTable<()>>>,
    cancellation_token: CancellationToken,
) {
    let state = ApiState {
        shared,
        whitelist_table,
        blacklist_table,
    };
    let with_access_update = Router::new()
        .route("/api/tunnel/access/whitelist", post(add_whitelist))
        .route(
            "/api/tunnel/access/whitelist/{id}",
            delete(remove_whitelist),
        )
        .route("/api/tunnel/access/blacklist", post(add_blacklist))
        .route(
            "/api/tunnel/access/blacklist/{id}",
            delete(remove_blacklist),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            access_middleware,
        ));

    let api = Router::new()
        .route("/api/tunnel/users", post(new_tunnel_user))
        .route("/api/tunnel/users/{id}", put(modify_tunnel_user_password))
        .route("/api/tunnel/users/{id}", delete(delete_tunnel_user))
        .route(
            "/api/tunnel/users/{id}/usage",
            get(get_tunnel_usage_by_user),
        )
        .route("/api/tunnel/users/{id}/token/rotate", post(rotate_token))
        .merge(with_access_update)
        .with_state(state);

    match TcpListener::bind(api_host).await {
        Ok(api_listener) => {
            select! {
                biased;
                _ = cancellation_token.cancelled() => { return; }
                serve_result = axum::serve(api_listener, api) => {
                    if let Err(error) = serve_result {
                        error!("Failed to start api services: {:?}", error);
                        return;
                    }
                }
            }
        }
        Err(error) => {
            error!("Failed to start api services: {:?}", error);
            return;
        }
    };
}

async fn access_middleware(
    State(api_state): State<ApiState>,
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;
    if let Err(error) = update_access_ip_tables(
        api_state.shared.db_connection,
        api_state.whitelist_table,
        api_state.blacklist_table,
    )
    .await
    {
        warn!("Unable to update the access tables: {:?}", error);
    }
    response
}
