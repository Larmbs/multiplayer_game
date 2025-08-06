//! # Minimal Async Server
//!
//! This crate defines a simple asynchronous server using Tokio.
//! It supports starting a server on a given address and printing the version.

use anyhow::Result;
use clap::Parser;

mod cli;
mod server;

use crate::cli::Cli;
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments (including --version support automatically)
    let cli = Cli::parse();

    let mut server = Server::init(&cli.address, cli.config).await?;
    println!("Started server, listening on {}.", server.get_address());
    server.run().await
}
