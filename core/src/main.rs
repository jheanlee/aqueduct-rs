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
use tokio::io;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;

mod common;
mod config;
mod core;
mod orm;

static LOG_CONFIG: LazyLock<Mutex<LogConfig>> = LazyLock::new(|| {
    Mutex::new(LogConfig {
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
        let mut log_config = LOG_CONFIG.lock().await;
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

    let mut threads = JoinSet::new();

    let tls_acceptor = TlsAcceptor::from(server_config);
    let tcp_listener = TcpListener::bind(config.tunnel_host).await?;

    log(
        Level::Notice,
        format!("Started listening on {}", config.tunnel_host.to_string()).as_str(),
        "core::main",
    )
    .await;

    loop {
        //  accept connection
        if let Ok((stream, client_addr)) = tcp_listener.accept().await {
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

                threads.spawn(tunnel_client_control(
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

    cancellation_token.cancel();
    threads.join_all().await;
}
