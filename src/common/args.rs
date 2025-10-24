#[derive(clap::Parser)]
pub struct Args {
  #[arg(short, long, default_value_t = { "0.0.0.0".to_string() })]
  pub host_addr: String,
  #[arg(short_alias = 'p', long, default_value_t = 51000)]
  pub host_port: u16,
}