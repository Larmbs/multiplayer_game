use clap::Parser;

/// Command-line arguments for the server application.
#[derive(Parser, Debug)]
#[command(name = "Server")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub address: String,

    #[command(flatten)]
    pub config: ServerConfig,
}

/// Configuration for the server, parsed from command-line arguments.
///
/// This struct defines the configurable parameters for the server
/// that can be set via CLI flags when launching the application.
#[derive(Debug, Parser)]
pub struct ServerConfig {
    #[arg(long, default_value = "New server")]
    pub server_name: String,

    #[arg(long)]
    pub password: Option<String>,

    /// Maximum number of clients allowed
    #[arg(long, default_value_t = 10)]
    pub max_clients: usize,

    /// Enable broadcasting server presence on UDP
    #[arg(long, default_value_t = false)]
    pub broadcast: bool,
}
