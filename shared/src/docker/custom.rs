use anyhow::{anyhow, Result};
use httparse::{Response, EMPTY_HEADER};
use serde::Deserialize;
use serde_json::json;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};

use super::DockerService;

#[derive(Debug)]
pub struct CustomApi {
    pub stream_path: String,
}

impl CustomApi {
    pub fn new() -> Self {
        let socket_path = "/var/run/docker.sock";
        CustomApi {
            stream_path: socket_path.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskStatus {
    #[serde(rename = "State")]
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskReponse {
    #[serde(rename = "Status")]
    pub status: TaskStatus,
    #[serde(rename = "DesiredState")]
    pub desired_state: String,
}

impl DockerService {
    pub async fn list_task_status(&self, service_name: &str) -> Result<Vec<TaskReponse>> {
        let filters = json!({
            "service": {
                service_name: true
            }
        });
        let filters_encoded = urlencoding::encode(filters.to_string().as_str()).to_string();
        let query = format!("/v1.47/tasks?filters={}", filters_encoded);
        let request = format!(
            "GET {} HTTP/1.1\r\n\
                   Host: localhost\r\n\
                   Connection: close\r\n\
                   \r\n",
            query
        );
        let mut stream = UnixStream::connect(&self.custom_api.stream_path).await?;
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.unwrap();
        let response = String::from_utf8_lossy(&response).to_string();
        let parts = response.splitn(2, "\r\n\r\n").collect::<Vec<_>>();
        let body_part = parts.get(1).unwrap_or(&"");

        let body = parse_body(body_part)?;

        let tasks = serde_json::from_str::<Vec<TaskReponse>>(&body)?;
        Ok(tasks)
    }
}

fn parse_body(body: &str) -> Result<String> {
    let parts = body.split('\n').collect::<Vec<_>>();
    if parts.len() >= 2 {
        Ok(parts[1].to_string())
    } else if parts.len() == 1 {
        Ok(parts[0].to_string())
    } else if parts.len() == 0 {
        Ok(String::new())
    } else {
        Err(anyhow!("Invalid response body"))
    }
}

#[tokio::test]
async fn custom_api() {
    let docker = DockerService::new().unwrap();
    let tasks = docker
        .list_task_status("exnext-frontend-service")
        .await
        .unwrap();
    for task in tasks {
        println!("task: {:#?}", task);
        assert_eq!(task.status.state, task.desired_state);
    }
}
