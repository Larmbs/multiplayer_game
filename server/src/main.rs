//! This binary is part of the multiplayer game project.
//! It defines the main entry point for the game server, which initializes the server with the specified
//! address and configuration, and runs the server to handle client connections and game logic.
use anyhow::Result;
use clap::Parser;

mod cli;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let mut server = server::Server::init(&cli.address, cli.config).await?;
    println!("Started server, listening on {}.", server.get_address().unwrap());
    server.run().await
}
