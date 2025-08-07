use clap::Parser;
use common::details;
/// Command-line arguments for the server application.
#[derive(Parser, Debug)]
#[command(name = "Client")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub address: String,

    #[arg(long, default_value = details::DEFAULT_USERNAME)]
    pub username: String,

    #[arg(long, default_value = None)]
    pub password: Option<String>,

    #[arg(long)]
    pub metal: bool,
}
