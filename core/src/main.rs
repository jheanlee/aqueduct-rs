use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use sea_orm::Database;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use crate::config::config_handler::read_config;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::{Flags, TunnelClient, TunnelStatus};

mod common;
mod core;
mod config;
mod orm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  let _ = dotenv::dotenv();
  let config = read_config().map_err(|error| { error.to_string() })?;

  //  database
  let db_connection = Database::connect(
    format!(
      "postgres://{}:{}@{}:{}/{}",
      config.db_username,
      config.db_password,
      config.db_host,
      config.db_port,
      config.db_name
    )
  ).await?;

  //  tls credentials
  let cert = CertificateDer::pem_file_iter(config.tls_cert_path)?
    .collect::<Result<Vec<_>, _>>()?;
  let key = PrivateKeyDer::from_pem_file(config.tls_private_key_path)?;
  let server_config = Arc::new(
    rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(cert, key).expect("TLS config error")
  );

  //  shared
  let cancellation_token = CancellationToken::new();
  let tunnel_status = Arc::new(TunnelStatus {
    host: config.tunnel_host.ip().to_string(),
    available_ports: RwLock::new(VecDeque::new()),
    proxy_queue: RwLock::new(HashMap::new()),
    db_connection: db_connection
  });

  let mut threads = JoinSet::new();

  let tls_acceptor = TlsAcceptor::from(server_config);
  let tcp_listener = TcpListener::bind(config.tunnel_host).await?;

  //  TODO log listen start
  
  loop {
    //  accept connection
    if let Ok((stream, client_addr)) = tcp_listener.accept().await {
      //  TODO check whitelist

      //  tls
      let tls_acceptor = tls_acceptor.clone();
      if let Ok(tls_stream) = tls_acceptor.accept(stream).await {

        // let (stream_reader, stream_writer) = io::split(tls_stream);

        threads.spawn(
          tunnel_client_control(
            Flags {
              global_cancellation_token: cancellation_token.clone(),
              local_cancellation_token: CancellationToken::new(),
            },
            Arc::new(TunnelClient {
              stream: Mutex::new(tls_stream),
              // stream_writer: Mutex::new(stream_writer),
              // stream_reader: Mutex::new(stream_reader),
              addr: client_addr,
            }),
            tunnel_status.clone()
          )
        );
      }
    }
  }

  cancellation_token.cancel();
  threads.join_all().await;
}
