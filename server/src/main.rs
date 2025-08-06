use anyhow::Result;
mod server;
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--version") | Some("-v") => {
            println!("Client version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(addr) => {
            let mut server = Server::init(addr).await?;
            println!("Started server, listening on {}.", server.get_address());
            server.run().await
        }
        None => {
            eprintln!("Usage: {} <server_address> [--version | -v]", args[0]);
            std::process::exit(1);
        }
    }
}
