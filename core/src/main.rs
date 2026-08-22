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
use crate::common::model::Shared;
use crate::common::signal_handler::signal_handler;
use crate::common::tunnel_info::TunnelInfo;
use crate::config::access_handler::update_access_ip_tables;
use crate::config::args::Commands;
use crate::config::config_handler::read_config;
use crate::config::db_config_handler::read_db_config;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::TunnelStatus;
use crate::core::tunnel::pending_cleaner::pending_client_cleaner;
use crate::orm::tunnel_session::database_tunnel_session_batch_task;
use crate::system_info::collector::{SystemInfo, system_info_cold, system_info_hot};
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
use tracing::{Instrument, debug, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(feature = "migration")]
use crate::config::args::MigrationModes;
use crate::init::runner::initialize_database;
use crate::orm::tunnel_status::database_tunnel_status_task;

#[cfg(feature = "api")]
mod api;
mod common;
mod config;
mod core;
pub mod init;
#[cfg(feature = "migration")]
mod migration;
mod orm;
mod system_info;

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
    let signal_handler_task = tokio::spawn(signal_handler(cancellation_token.clone()));

    //  database connection
    let mut db_connect_options = ConnectOptions::new(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password, config.db_host, config.db_port, config.db_name
    ));
    db_connect_options.sqlx_logging(false);
    let db_connection = Database::connect(db_connect_options)
        .await
        .unwrap_or_else(|error| {
            error!("Unable to connect to the database: {:?}", error);
            exit(1);
        });

    //  subcommands
    match config.subcommand {
        #[cfg(feature = "migration")]
        Some(Commands::Migrate(args)) => match args.mode {
            MigrationModes::Up => {
                use crate::migration::runner::migrate;
                let result = migrate(db_connection).await;
                match result {
                    Ok(_) => exit(0),
                    Err(_) => exit(1),
                }
            }
        },
        Some(Commands::Init) => {
            let result = initialize_database(db_connection).await;
            match result {
                Ok(_) => exit(0),
                Err(_) => exit(1),
            }
        }
        None => {} // _ => unreachable!(),
    }

    //  retrieve configuration stored in the database
    let db_config = read_db_config(&db_connection)
        .await
        .unwrap_or_else(|error| {
            error!("{}", error);
            exit(1);
        });

    //  whitelist and blacklist
    let whitelist = Arc::new(RwLock::new(IpNetworkTable::new()));
    let blacklist = Arc::new(RwLock::new(IpNetworkTable::new()));
    update_access_ip_tables(&db_connection, whitelist.clone(), blacklist.clone())
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
        host: config.tunnel_bind_address.ip().to_string(),
        available_ports: RwLock::new(config.tunnel_allowed_ports),
        pending_external_clients: DashMap::new(),
        client_connection_limit: config.tunnel_client_connection_limit,
    });

    //  shared
    let (database_tunnel_session_batch_tx, database_tunnel_session_batch_rx) = mpsc::channel(2048);
    let shared = Shared {
        db_connection: db_connection.clone(),
        auth_manager,
        database_tunnel_session_batch_tx,
    };

    //  pending external client cleaner
    let pending_cleaner_task = tokio::spawn(pending_client_cleaner(
        cancellation_token.clone(),
        tunnel_status.clone(),
    ));

    //  system info
    #[cfg(target_os = "linux")]
    sysinfo::set_open_files_limit(0);
    let system_info = Arc::new(SystemInfo {
        cpu_usage: Default::default(),
        used_memory: Default::default(),
        total_memory: Default::default(),
        process_cpu_usage: Default::default(),
        process_memory: Default::default(),
        process_fd_count: Default::default(),
    });
    let system_info_hot_task = tokio::spawn(system_info_hot(
        system_info.clone(),
        cancellation_token.clone(),
    ));
    let system_info_cold_task = tokio::spawn(system_info_cold(
        system_info.clone(),
        cancellation_token.clone(),
    ));

    //  tunnel info
    let tunnel_info = Arc::new(TunnelInfo {
        active_service_count: Default::default(),
        active_external_connection_count: Default::default(),
    });

    let database_tunnel_status_task = tokio::spawn(database_tunnel_status_task(
        db_connection.clone(),
        tunnel_info.clone(),
        cancellation_token.clone(),
    ));

    //  api
    #[cfg(feature = "api")]
    let api_task = {
        use crate::api::control::{ApiState, api_control};
        use crate::api::jwt::key::init_jwt_keys;
        use tokio::time::Instant;

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

        let api_state = Arc::new(ApiState {
            start_time: Instant::now(),
            shared: shared.clone(),
            system_info,
            tunnel_info: tunnel_info.clone(),
            whitelist_table: whitelist.clone(),
            blacklist_table: blacklist.clone(),
            jti_map: DashMap::new(),
            refresh_token_keys,
            access_token_keys,
        });

        tokio::spawn(api_control(
            config.api_bind_address,
            api_state,
            cancellation_token.clone(),
        ))
    };

    //  tunnel
    let mut tunnel_tasks = JoinSet::new();
    let database_tunnel_sessions_batch_task = tokio::spawn(database_tunnel_session_batch_task(
        shared.db_connection.clone(),
        database_tunnel_session_batch_rx,
        cancellation_token.clone(),
    ));

    let tls_acceptor = TlsAcceptor::from(server_config);
    let tcp_listener = TcpListener::bind(config.tunnel_bind_address)
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

    info!("Listening on {}", config.tunnel_bind_address.to_string());

    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => { break; }
            _ = tunnel_tasks.join_next(), if !tunnel_tasks.is_empty() => {}
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
                    let tunnel_info_clone = tunnel_info.clone();
                    let global_connection_semaphore_clone = global_connection_semaphore.clone();

                    tunnel_tasks.spawn(
                        async move {
                            match timeout(Duration::from_millis(5000), tls_acceptor_clone.accept(stream)).await {
                                Ok(Ok(tls_stream)) => {
                                    debug!("Connection accepted");

                                    tunnel_client_control(
                                        shared_clone,
                                        tls_stream,
                                        client_addr,
                                        tunnel_status_clone,
                                        tunnel_info_clone,
                                        global_connection_semaphore_clone,
                                        local_cancellation_token
                                    ).await;
                                }
                                Ok(Err(error)) => {
                                    debug!("TLS handshake failed: {:?}", error);
                                }
                                Err(_) => {
                                    debug!("TLS handshake timed out");
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
    info!("Shutdown signal received. Cleaning up");
    let _ = signal_handler_task.await;
    let _ = system_info_hot_task.await;
    let _ = system_info_cold_task.await;
    #[cfg(feature = "api")]
    let _ = api_task.await;
    let _ = database_tunnel_sessions_batch_task.await;
    let _ = database_tunnel_status_task.await;
    let _ = pending_cleaner_task.await;
    tunnel_tasks.join_all().await;
    info!("Shutdown complete");

    Ok(())
}
