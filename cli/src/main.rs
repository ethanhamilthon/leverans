use std::process::exit;

use clap::Parser;
use commands::Lev;
use routes::handle_routes;

pub mod api;
pub mod commands;
pub mod data;
pub mod handlers;
pub mod routes;
pub mod utils;

#[tokio::main]
async fn main() {
    let cli = Lev::parse();
    match handle_routes(cli).await {
        Ok(_) => exit(0),
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    }
}
