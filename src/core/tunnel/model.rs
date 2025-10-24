use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::{watch, Mutex};
use tokio_rustls::server::TlsStream;

#[derive(Clone)]
pub struct Flags {
  pub global_kill_rx: watch::Receiver<bool>,
  pub client_kill_rx: watch::Receiver<bool>,
  pub client_kill_tx: watch::Sender<bool>,
}

pub struct TunnelClient {
  pub stream: Mutex<TlsStream<TcpStream>>,
  // pub stream_writer: Mutex<WriteHalf<TlsStream<TcpStream>>>,
  // pub stream_reader: Mutex<ReadHalf<TlsStream<TcpStream>>>,
  pub addr: SocketAddr
}