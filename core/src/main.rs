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

use crate::common::log::{Level, LogConfig, log};
use crate::config::config_handler::read_config;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::{Flags, TunnelClient, TunnelStatus};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use sea_orm::Database;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, LazyLock};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use tokio::{io, select};
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
        syslog_enabled: false,
        oslog_enabled: false,
    })
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let _ = dotenv::dotenv();
    let config = read_config().map_err(|error| error.to_string())?;

    //  log
    {
        let mut log_config = LOG_CONFIG.write().await;
        *log_config.deref_mut() = config.log_config;
    }

    //  database
    let db_connection = Database::connect(format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_username, config.db_password, config.db_host, config.db_port, config.db_name
    ))
    .await?;

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

    //  shared
    let cancellation_token = CancellationToken::new();
    let tunnel_status = Arc::new(TunnelStatus {
        host: config.tunnel_host.ip().to_string(),
        available_ports: RwLock::new(config.tunnel_allowed_ports),
        proxy_queue: RwLock::new(HashMap::new()),
        db_connection: db_connection,
    });

    let mut tunnel_threads = JoinSet::new();

    let tls_acceptor = TlsAcceptor::from(server_config);
    let tcp_listener = TcpListener::bind(config.tunnel_host).await?;

    log(
        Level::Notice,
        format!("Started listening on {}", config.tunnel_host.to_string()).as_str(),
        "core::main",
    )
    .await;

    loop {
        select! {
            biased;
            _ = cancellation_token.cancelled() => { break; }
            _ = tunnel_threads.join_next(), if !tunnel_threads.is_empty() => {}
            accept_result = tcp_listener.accept() => {
                //  accept connection
                if let Ok((stream, client_addr)) = accept_result {
                    //  TODO check whitelist

                    //  tls
                    let tls_acceptor = tls_acceptor.clone();
                    if let Ok(tls_stream) = tls_acceptor.accept(stream).await {
                        log(
                            Level::Info,
                            format!("Connection accepted from {}", client_addr.to_string()).as_str(),
                            "core::main",
                        )
                        .await;

                        let (stream_rx, stream_tx) = io::split(tls_stream);

                        tunnel_threads.spawn(tunnel_client_control(
                            Flags {
                                global_cancellation_token: cancellation_token.clone(),
                                local_cancellation_token: CancellationToken::new(),
                            },
                            Arc::new(TunnelClient {
                                stream_tx: Mutex::new(stream_tx),
                                stream_rx: Mutex::new(stream_rx),
                                addr: client_addr,
                            }),
                            tunnel_status.clone(),
                        ));
                    }
                }
            }
        }
    }

    cancellation_token.cancel();
    tunnel_threads.join_all().await;
    Ok(())
}
