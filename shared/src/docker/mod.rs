use std::sync::Arc;

use anyhow::Result;
use bollard::Docker;
pub mod custom;
pub mod image;
pub mod service;
pub mod volume;

#[derive(Debug, Clone)]
pub struct DockerService {
    conn: Arc<Docker>,
    custom_api: Arc<custom::CustomApi>,
}

impl DockerService {
    pub fn new() -> Result<Self> {
        Ok(DockerService {
            custom_api: Arc::new(custom::CustomApi::new()),
            conn: Arc::new(Docker::connect_with_socket_defaults()?),
        })
    }
}

#[tokio::test]
async fn test_connect() -> Result<()> {
    let start = std::time::Instant::now();
    let _ = DockerService::new()?;
    println!("new docker: {:?}", start.elapsed());
    Ok(())
}
