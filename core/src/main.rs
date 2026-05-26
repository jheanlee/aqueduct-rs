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
use crate::api::jwt::key::init_jwt_keys;
use crate::common::auth_manager::AuthManager;
use crate::common::model::Shared;
use crate::common::signal_handler::signal_handler;
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
use sea_orm::{ConnectOptions, Database};
use socket2::{SockRef, TcpKeepalive};
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

mod api;
mod common;
mod config;
mod core;
mod orm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let _ = dotenv::dotenv();
    let config = read_config().unwrap_or_else(|error| {
        println!("{}", error);
        exit(1);
    });
    let cancellation_token = CancellationToken::new();

    // crypto
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    //  log
    let (non_blocking_stdout, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking_stdout)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    subscriber.init();

    //  signal handler
    let signal_handler_thread = tokio::spawn(signal_handler(cancellation_token.clone()));

    //  database
    let mut db_connect_options = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_username, config.db_password, config.db_host, config.db_port, config.db_name
    ));
    db_connect_options.sqlx_logging(false);
    let db_connection = Database::connect(db_connect_options)
        .await
        .unwrap_or_else(|error| {
            error!("Unable to connect to the database: {:?}", error);
            exit(1);
        });
    let db_config = read_db_config(db_connection.clone())
        .await
        .unwrap_or_else(|error| {
            error!("{}", error);
            exit(1);
        });

    //  whitelist and blacklist
    let whitelist = Arc::new(RwLock::new(IpNetworkTable::new()));
    let blacklist = Arc::new(RwLock::new(IpNetworkTable::new()));
    update_access_ip_tables(db_connection.clone(), whitelist.clone(), blacklist.clone())
        .await
        .unwrap_or_else(|error| {
            error!("{}", error);
            exit(1);
        });

    //  tls credentials
    let cert = CertificateDer::pem_file_iter(config.tls_cert_path)?
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| {
            error!("Unable to retrieve the TLS certificate: {:?}", error);
            exit(1);
        });
    let key = PrivateKeyDer::from_pem_file(config.tls_private_key_path).unwrap_or_else(|error| {
        error!("Unable to retrieve the TLS private key: {:?}", error);
        exit(1);
    });
    let server_config = Arc::new(
        rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .unwrap_or_else(|error| {
                error!("TLS config error: {:?}", error);
                exit(1);
            }),
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
    let refresh_token_keys = match init_jwt_keys(
        config.jwt_refresh_private_key_path,
        config.jwt_refresh_public_key_path,
    )
    .await
    {
        Ok(keys) => keys,
        Err(error) => {
            error!(
                "Unable to initialise refresh token signing keys: {:?}",
                error
            );
            exit(1);
        }
    };
    let access_token_keys = match init_jwt_keys(
        config.jwt_access_private_key_path,
        config.jwt_access_public_key_path,
    )
    .await
    {
        Ok(keys) => keys,
        Err(error) => {
            error!(
                "Unable to initialise refresh token signing keys: {:?}",
                error
            );
            exit(1);
        }
    };
    let api_thread = tokio::spawn(api_control(
        config.api_host,
        shared.clone(),
        whitelist.clone(),
        blacklist.clone(),
        refresh_token_keys,
        access_token_keys,
        cancellation_token.clone(),
    ));

    //  tunnel
    let mut tunnel_threads = JoinSet::new();
    let database_tunnel_sessions_batch_thread = tokio::spawn(database_tunnel_session_batch_thread(
        shared.db_connection.clone(),
        database_tunnel_session_batch_rx,
        cancellation_token.clone(),
    ));

    let tls_acceptor = TlsAcceptor::from(server_config);
    let tcp_listener = TcpListener::bind(config.tunnel_host)
        .await
        .unwrap_or_else(|error| {
            error!("Unable to bind the control listener: {:?}", error);
            exit(1);
        });
    let socket = SockRef::from(&tcp_listener);
    socket.listen(4096).unwrap_or_else(|error| {
        error!("Unable to configure the control listener: {:?}", error);
        exit(1);
    });
    socket.set_reuse_address(true).unwrap_or_else(|error| {
        error!("Unable to configure the control listener: {:?}", error);
        exit(1);
    });
    socket.set_tcp_nodelay(true).unwrap_or_else(|error| {
        error!("Unable to configure the control listener: {:?}", error);
        exit(1);
    });

    warn!("Started listening on {}", config.tunnel_host.to_string());

    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => { break; }
            _ = tunnel_threads.join_next(), if !tunnel_threads.is_empty() => {}
            accept_result = tcp_listener.accept() => {
                //  accept connection
                if let Ok((stream, client_addr)) = accept_result {
                    let session_span = info_span!("session", tunnel_client = %client_addr);

                    let socket_ref = SockRef::from(&stream);
                    let socket_keep_alive = TcpKeepalive::new()
                        .with_time(Duration::from_secs(60))
                        .with_interval(Duration::from_secs(30))
                        .with_retries(3);
                    socket_ref.set_tcp_keepalive(&socket_keep_alive).unwrap_or_else(|error| {
                        warn!("Unable to configure socket: {:?}", error);
                    });

                    //  whitelist & blacklist
                    if db_config.whitelist && whitelist.read().await.matches(client_addr.ip()).count() == 0 {
                        drop(stream);
                        session_span.in_scope(|| {info!("Access denied (whitelist)")});
                        continue;
                    }

                    if db_config.blacklist && blacklist.read().await.matches(client_addr.ip()).count() > 0 {
                        drop(stream);
                        session_span.in_scope(|| {info!("Access denied (blacklist)")});
                        continue;
                    }

                    //  tls
                    let tls_acceptor_clone = tls_acceptor.clone();
                    let local_cancellation_token = cancellation_token.child_token();
                    let shared_clone = shared.clone();
                    let tunnel_status_clone = tunnel_status.clone();
                    let global_connection_semaphore_clone = global_connection_semaphore.clone();

                    tunnel_threads.spawn(
                        async move {
                            match timeout(Duration::from_millis(5000), tls_acceptor_clone.accept(stream)).await {
                                Ok(Ok(tls_stream)) => {
                                    info!("Session started");

                                    tunnel_client_control(
                                        Flags {
                                            local_cancellation_token,
                                        },
                                        shared_clone,
                                        tls_stream,
                                        client_addr,
                                        tunnel_status_clone,
                                        global_connection_semaphore_clone
                                    ).await;
                                }
                                Ok(Err(error)) => {
                                    warn!("TLS handshake failed: {:?}", error);
                                }
                                Err(_) => {
                                    warn!("TLS handshake timed out");
                                }
                            }
                        }
                        .instrument(session_span)
                    );
                }
            }
        }
    }

    cancellation_token.cancel();
    warn!("Exit signal received. Cleaning up");
    let _ = signal_handler_thread.await;
    let _ = api_thread.await;
    let _ = database_tunnel_sessions_batch_thread.await;
    let _ = pending_cleaner_thread.await;
    tunnel_threads.join_all().await;
    warn!("Finished cleaning up");

    Ok(())
}
