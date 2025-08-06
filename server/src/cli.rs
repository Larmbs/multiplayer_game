use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Server")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub address: String,

    #[command(flatten)]
    pub config: ServerConfig,
}

#[derive(Debug, Parser)]
pub struct ServerConfig {
    #[arg(long, default_value = "New server")]
    pub server_name: String,

    #[arg(long)]
    pub password: Option<String>,

    #[arg(long, default_value_t = 10)]
    pub max_clients: usize,

    /// Enable broadcasting server presence on UDP
    #[arg(long, default_value_t = false)]
    pub broadcast: bool,
}
