use std::error::Error;

use rand::{distributions::Alphanumeric, Rng};
use server::{auth_handler::change_jwt_secret, start_server, ServerData};

pub mod cron;
pub mod repo;
pub mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    onstart();
    let sr = ServerData::new(8081).await;
    start_server(sr).await?;
    Ok(())
}

fn onstart() {
    println!("Starting server...");
    println!("Listening on 0.0.0.0:8081");

    println!("trying to get JWT_KEY env var");
    let key = std::env::var("JWT_KEY").unwrap_or(generate_random_string(32));
    change_jwt_secret(key.as_str());
}
fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}
