use crate::docker::DockerService;
use anyhow::Result;

use super::deploy::DeployTask;

pub async fn handle_deploy_tasks(tasks: Vec<DeployTask>, docker: DockerService) -> Result<()> {
    for task in tasks {
        match task {
            DeployTask::Build(_) => continue,
            DeployTask::HealthCheck => {}
        }
    }
    Ok(())
}
