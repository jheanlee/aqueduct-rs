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
use crate::common::auth_manager::AuthManager;
use crate::orm::tunnel_session::DatabaseTunnelSessionAction;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Shared {
    pub db_connection: DatabaseConnection,
    pub auth_manager: Arc<AuthManager>,
    pub database_tunnel_session_batch_tx: mpsc::Sender<DatabaseTunnelSessionAction>,
}
