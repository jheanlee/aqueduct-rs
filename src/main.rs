use std::sync::Arc;
use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use tokio::net::TcpListener;
use tokio::sync::{watch, Mutex};
use tokio_rustls::TlsAcceptor;
use crate::common::args::Args;
use crate::core::tunnel::control::tunnel_client_control;
use crate::core::tunnel::model::{Flags, TunnelClient};

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

  let config = rustls::ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(cert, key)?;
  let tls_acceptor = TlsAcceptor::from(Arc::new(config));
  let tcp_listener = TcpListener::bind(format!("{}:{}", args.host_addr, args.host_port)).await?;

  let (global_kill_tx, global_kill_rx) = watch::channel(false);
  
  loop {
    let (stream, client_addr) = tcp_listener.accept().await?;
    let tls_acceptor = tls_acceptor.clone();
    let tls_stream = tls_acceptor.accept(stream).await?;  //  TODO check error values
    // let (stream_reader, stream_writer) = io::split(tls_stream);

    let (client_kill_tx, client_kill_rx) = watch::channel(false);

    let _tunnel_client_control_thread = tokio::spawn(
      tunnel_client_control(
        Flags {
          global_kill_rx: global_kill_rx.clone(),
          client_kill_rx,
          client_kill_tx,
        },
        Arc::new(TunnelClient { 
          stream: Mutex::new(tls_stream), 
          // stream_writer: Mutex::new(stream_writer), 
          // stream_reader: Mutex::new(stream_reader), 
          addr: client_addr
        })
      )
    );
  }
}
