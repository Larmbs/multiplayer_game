//! This file is part of the multiplayer game project.
//! It defines the command-line interface (CLI) for the game server, allowing users to specify
//! the server address, configuration options, and other parameters when starting the server.
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "Server")]
pub struct Cli {
    pub address: String,

    #[command(flatten)]
    pub config: ServerConfig,
}

#[derive(Debug, Parser)]
pub struct ServerConfig {
    #[arg(long, default_value = "New Server")]
    pub server_name: String,

    #[arg(long)]
    pub password: Option<String>,

    #[arg(long, default_value_t = 10)]
    pub max_clients: usize,
}
