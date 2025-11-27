use std::net::SocketAddr;

#[derive(clap::Parser)]
pub struct Args {
  pub host: Option<SocketAddr>,
  #[arg(long)]
  pub tls_cert: Option<String>,
  #[arg(long)]
  pub tls_private_key: Option<String>
}