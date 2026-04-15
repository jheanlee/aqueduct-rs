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
    pub stream_tx: Mutex<WriteHalf<TlsStream<TcpStream>>>,
    pub stream_rx: Mutex<ReadHalf<TlsStream<TcpStream>>>,
    pub addr: SocketAddr,
}

pub struct TunnelStatus {
    pub host: String,
    pub available_ports: RwLock<VecDeque<u16>>,
    pub proxy_queue: RwLock<HashMap<String, ProxyClient>>,
}

pub struct ProxyClient {
    pub proxy_id: String,
    pub tunnel_client_user_id: String,
    pub external_client_stream_rx: tcp::OwnedReadHalf,
    pub external_client_stream_tx: tcp::OwnedWriteHalf,
    pub external_client_addr: SocketAddr,
    pub proxy_control_client_addr: SocketAddr,
    pub proxy_control_server_addr: SocketAddr,
}
