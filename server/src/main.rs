use anyhow::Result;
use clap::Parser;

mod cli;
mod server;

use crate::cli::Cli;
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut server = Server::init(&cli.address, cli.config).await?;
    println!("Started server, listening on {}.", server.get_address());
    server.run().await
}
