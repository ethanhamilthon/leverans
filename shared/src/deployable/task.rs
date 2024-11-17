use std::time::Duration;

use crate::docker::DockerService;
use anyhow::{anyhow, Result};
use tokio::time::sleep;

use super::deploy::DeployTask;

pub async fn handle_deploy_tasks(tasks: Vec<DeployTask>, docker: DockerService) -> Result<()> {
    for task in tasks {
        run_deploy_task(task, docker.clone()).await?;
    }
    Ok(())
}

pub async fn run_deploy_task(task: DeployTask, docker: DockerService) -> Result<()> {
    match task {
        DeployTask::Build(_) => Ok(()),
        DeployTask::HealthCheck(health) => {
            println!("health check: {}", health.service_name);
            let mut retrys = health.timeout_sec * 5;
            let mut waits = health.wait_sec * 5;
            loop {
                sleep(Duration::from_millis(200)).await;
                if waits != 0 {
                    waits -= 1;
                    continue;
                }
                if retrys == 0 {
                    return Err(anyhow!("health check timeout"));
                }
                retrys -= 1;
                let tasks = docker.list_task_status(&health.service_name).await?;
                let mut is_ok = true;
                for task in tasks {
                    if task.status.state != task.desired_state {
                        is_ok = false;
                    }
                }
                if is_ok {
                    break;
                }
            }
            Ok(())
        }
    }
}
