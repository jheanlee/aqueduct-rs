use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;
use crate::common::args::Args;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::{Flags, TunnelClient, TunnelStatus};

mod common;
mod core;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  let _ = dotenv::dotenv();
  let args = Args::parse();

  let cert_path = std::env::var("AQUEDUCT_TLS_CERT").expect("TLS certificate not set");
  let private_key_path = std::env::var("AQUEDUCT_TLS_KEY").expect("TLS private key not set");
  let cert = CertificateDer::pem_file_iter(cert_path)?
    .collect::<Result<Vec<_>, _>>()?;
  let key = PrivateKeyDer::from_pem_file(private_key_path)?;

  let config = Arc::new(rustls::ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(cert, key)?);
  let tls_acceptor = TlsAcceptor::from(config);
  let tcp_listener = TcpListener::bind(format!("{}:{}", args.host_addr, args.host_port)).await?;

  let cancellation_token = CancellationToken::new();
  let tunnel_status = Arc::new(TunnelStatus {
    host: args.host_addr,
    available_ports: RwLock::new(VecDeque::new()),
    proxy_queue: RwLock::new(HashMap::new()),
  });
  let mut threads = JoinSet::new();
  
  loop {
    let (stream, client_addr) = tcp_listener.accept().await?;
    let tls_acceptor = tls_acceptor.clone();
    let tls_stream = tls_acceptor.accept(stream).await?;  //  TODO check error values
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

  cancellation_token.cancel();
  threads.join_all().await;
}
