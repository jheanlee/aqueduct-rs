use std::net::SocketAddr;

#[derive(clap::Parser)]
pub struct Args {
  pub host: Option<SocketAddr>,
  #[arg(long)]
  pub tls_cert: Option<String>,
  #[arg(long)]
  pub tls_private_key: Option<String>,
  #[arg(long)]
  pub db_name: Option<String>,
  #[arg(long)]
  pub db_host: Option<String>,
  #[arg(long)]
  pub db_port: Option<u16>,
  #[arg(long)]
  pub db_username: Option<String>,
  #[arg(long)]
  pub db_password: Option<String>,
  #[arg(long)]
  pub daemon: Option<bool>,
  #[arg(long)]
  pub stdout_filter: Option<u8>,
  #[arg(long)]
  pub log_filter: Option<u8>
}