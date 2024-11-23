use std::time::Duration;

use crate::docker::DockerService;
use anyhow::{anyhow, Result};
use bollard::secret::ServiceUpdateStatusStateEnum;
use tokio::time::sleep;

use super::deploy::DeployTask;

pub async fn handle_deploy_tasks(tasks: Vec<DeployTask>, docker: DockerService) -> Result<()> {
    for task in tasks {
        run_deploy_task(task, docker.clone()).await?;
    }
    Ok(())
}

pub async fn run_deploy_task(task: DeployTask, docker: DockerService) -> Result<()> {
    let health_check_task = match task {
        DeployTask::Build(_) => return Ok(()),
        DeployTask::HealthCheck(health_checkable) => health_checkable,
    };
    println!(
        "waiting for health check: {}",
        health_check_task.service_name
    );
    sleep(Duration::from_millis(
        (1000 * health_check_task.wait_sec) as u64,
    ))
    .await;
    loop {
        let service = docker
            .get_service(health_check_task.service_name.clone())
            .await?;
        if service.update_status.is_none() {
            break;
        } else {
            let update_status = service.update_status.unwrap();
            if update_status.state.is_some()
                && update_status.state.unwrap() == ServiceUpdateStatusStateEnum::COMPLETED
            {
                println!("health check passed: {}", health_check_task.service_name);
                break;
            }

            if update_status.state.is_some()
                && update_status.state.unwrap() == ServiceUpdateStatusStateEnum::PAUSED
            {
                return Err(anyhow!(
                    "health check failed: {}",
                    health_check_task.service_name
                ));
            }

            if update_status.state.is_none() {
                break;
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
    Ok(())
}
