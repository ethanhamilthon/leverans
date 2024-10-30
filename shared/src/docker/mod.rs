use anyhow::Result;
use bollard::Docker;
pub mod image;
pub mod service;
pub mod volume;

#[derive(Debug, Clone)]
pub struct DockerService {
    conn: Docker,
}

impl DockerService {
    pub fn new() -> Result<Self> {
        println!("Connecting docker");
        Ok(DockerService {
            conn: Docker::connect_with_socket_defaults()?,
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
