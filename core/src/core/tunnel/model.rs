use sea_orm::DatabaseConnection;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::{TcpStream, tcp};
use tokio::sync::{Mutex, RwLock};
use tokio_rustls::server::TlsStream;
use tokio_util::sync::CancellationToken;

pub enum ClientType {
    Service,
    Proxy,
}

#[derive(Clone)]
pub struct Flags {
    pub global_cancellation_token: CancellationToken,
    pub local_cancellation_token: CancellationToken,
}

pub struct TunnelClient {
    // pub stream: Mutex<TlsStream<TcpStream>>,
    pub stream_tx: Mutex<WriteHalf<TlsStream<TcpStream>>>,
    pub stream_rx: Mutex<ReadHalf<TlsStream<TcpStream>>>,
    pub addr: SocketAddr,
}

pub struct TunnelStatus {
    pub host: String,
    pub available_ports: RwLock<VecDeque<u16>>,
    pub proxy_queue: RwLock<HashMap<String, ProxyClient>>,
    pub db_connection: DatabaseConnection,
}

pub struct ProxyClient {
    pub external_client_stream_rx: tcp::OwnedReadHalf,
    pub external_client_stream_tx: tcp::OwnedWriteHalf,
    pub external_client_addr: SocketAddr,
    pub proxy_control_client_addr: SocketAddr,
    pub proxy_control_server_addr: SocketAddr,
}
