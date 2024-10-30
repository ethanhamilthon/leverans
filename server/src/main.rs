use std::error::Error;

use server::{start_server, ServerData};

pub mod repo;
pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let sr = ServerData::new(8081).await;
    start_server(sr).await?;
    Ok(())
}
