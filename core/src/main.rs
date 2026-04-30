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
use crate::api::control::api_control;
use crate::common::auth_manager::AuthManager;
use crate::common::log::{Level, LogConfig, log};
use crate::common::model::Shared;
use crate::config::access_handler::update_access_ip_tables;
use crate::config::config_handler::read_config;
use crate::config::db_config_handler::read_db_config;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::{Flags, TunnelStatus};
use crate::core::tunnel::pending_cleaner::pending_client_cleaner;
use crate::orm::tunnel_session::database_tunnel_session_batch_thread;
use dashmap::DashMap;
use ip_network_table::IpNetworkTable;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sea_orm::Database;
use socket2::{SockRef, TcpKeepalive};
use std::process::exit;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;

mod api;
mod common;
mod config;
mod core;
mod orm;

static LOG_CONFIG: LazyLock<RwLock<LogConfig>> = LazyLock::new(|| {
    RwLock::new(LogConfig {
        stdout_filter: Level::Info.into(),
        system_filter: Level::Notice.into(),
        stdout_enabled: true,
        #[cfg(target_os = "linux")]
        syslog_enabled: false,
        #[cfg(target_os = "macos")]
        oslog_enabled: false,
    })
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let _ = dotenv::dotenv();
    let config = read_config().unwrap_or_else(|error| {
        println!("{}", error);
        exit(1);
    });
    let cancellation_token = CancellationToken::new();

    //  log
    {
        let mut log_config = LOG_CONFIG.write().await;
        *log_config = config.log_config;
    }

    //  database
    let db_connection = Database::connect(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_username, config.db_password, config.db_host, config.db_port, config.db_name
    ))
    .await?;
    let db_config = read_db_config(db_connection.clone())
        .await
        .unwrap_or_else(|error| {
            println!("{}", error);
            exit(1);
        });

    //  whitelist and blacklist
    let whitelist = Arc::new(RwLock::new(IpNetworkTable::new()));
    let blacklist = Arc::new(RwLock::new(IpNetworkTable::new()));
    update_access_ip_tables(db_connection.clone(), whitelist.clone(), blacklist.clone()).await?;

    //  tls credentials
    let cert =
        CertificateDer::pem_file_iter(config.tls_cert_path)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(config.tls_private_key_path)?;
    let server_config = Arc::new(
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .expect("TLS config error"),
    );

    //  auth
    let auth_manager = Arc::new(AuthManager::new());

    //  tunnel shared
    let global_connection_semaphore = Arc::new(Semaphore::new(
        config.tunnel_global_connection_limit as usize,
    ));
    let tunnel_status = Arc::new(TunnelStatus {
        host: config.tunnel_host.ip().to_string(),
        available_ports: RwLock::new(config.tunnel_allowed_ports),
        pending_external_clients: DashMap::new(),
        client_connection_limit: config.tunnel_client_connection_limit,
    });

    //  shared
    let (database_tunnel_session_batch_tx, database_tunnel_session_batch_rx) = mpsc::channel(2048);
    let shared = Shared {
        db_connection,
        auth_manager,
        database_tunnel_session_batch_tx,
    };

    //  pending external client cleaner
    let pending_cleaner_thread = tokio::spawn(pending_client_cleaner(
        cancellation_token.clone(),
        tunnel_status.clone(),
    ));

    //  api
    let api_thread = tokio::spawn(api_control(
        config.api_host,
        shared.clone(),
        whitelist.clone(),
        blacklist.clone(),
        cancellation_token.clone(),
    ));

    //  tunnel
    let mut tunnel_threads = JoinSet::new();
    let database_tunnel_sessions_batch_thread = tokio::spawn(database_tunnel_session_batch_thread(
        shared.clone(),
        database_tunnel_session_batch_rx,
        cancellation_token.clone(),
    ));

    let tls_acceptor = TlsAcceptor::from(server_config);
    let tcp_listener = TcpListener::bind(config.tunnel_host).await?;
    let socket = SockRef::from(&tcp_listener);
    socket.listen(4096)?;
    socket.set_reuse_address(true)?;
    socket.set_tcp_nodelay(true)?;

    log(
        Level::Notice,
        format!("Started listening on {}", config.tunnel_host.to_string()).as_str(),
        "core::main",
    )
    .await;

    loop {
        select! {
            _ = cancellation_token.cancelled() => { break; }
            _ = tunnel_threads.join_next(), if !tunnel_threads.is_empty() => {}
            accept_result = tcp_listener.accept() => {
                //  accept connection
                if let Ok((stream, client_addr)) = accept_result {
                    let socket_ref = SockRef::from(&stream);
                    let socket_keep_alive = TcpKeepalive::new()
                        .with_time(Duration::from_secs(60))
                        .with_interval(Duration::from_secs(30))
                        .with_retries(3);
                    socket_ref.set_tcp_keepalive(&socket_keep_alive)?;

                    //  whitelist & blacklist
                    if db_config.whitelist && whitelist.read().await.matches(client_addr.ip()).count() == 0 {
                        drop(stream);
                        log(
                            Level::Info,
                            format!("Access from {} denied (whitelist)", client_addr.to_string()).as_str(),
                            "core::main",
                        )
                        .await;
                        continue;
                    }

                    if db_config.blacklist && blacklist.read().await.matches(client_addr.ip()).count() > 0 {
                        drop(stream);
                        log(
                            Level::Info,
                            format!("Access from {} denied (blacklist)", client_addr.to_string()).as_str(),
                            "core::main",
                        )
                        .await;
                        continue;
                    }

                    //  tls
                    let tls_acceptor_clone = tls_acceptor.clone();
                    let cancellation_token_clone = cancellation_token.clone();
                    let shared_clone = shared.clone();
                    let tunnel_status_clone = tunnel_status.clone();
                    let global_connection_semaphore_clone = global_connection_semaphore.clone();

                    tunnel_threads.spawn(async move {
                        match timeout(Duration::from_millis(5000), tls_acceptor_clone.accept(stream)).await {
                            Ok(Ok(tls_stream)) => {
                                log(
                                    Level::Info,
                                    format!("Connection from {} accepted", client_addr.to_string()).as_str(),
                                    "core::main",
                                )
                                .await;

                                tunnel_client_control(
                                    Flags {
                                        global_cancellation_token: cancellation_token_clone,
                                        local_cancellation_token: CancellationToken::new(),
                                    },
                                    shared_clone,
                                    tls_stream,
                                    client_addr,
                                    tunnel_status_clone,
                                    global_connection_semaphore_clone
                                ).await;
                            }
                            _ => {}
                        }
                    });
                }
            }
        }
    }

    cancellation_token.cancel();
    let _ = api_thread.await;
    let _ = database_tunnel_sessions_batch_thread.await;
    let _ = pending_cleaner_thread.await;
    tunnel_threads.join_all().await;

    Ok(())
}
