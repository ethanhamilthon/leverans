use std::{
    fs,
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use shared::{
    config::MainConfig, docker::DockerService, docker_platform::get_docker_platform, err, ok,
};
use tokio::time::sleep;

use crate::{
    api::API,
    handlers::build_handle::{new_build_images, upload_images},
};

use super::plan_handle::handle_plan;

pub async fn new_handle_deploy(
    file_name: String,
    context: String,
    no_build: bool,
    filter: Option<String>,
    only: Option<Vec<String>>,
) -> Result<()> {
    let (user, deploys) = handle_plan(filter, only, file_name, context.clone(), no_build).await?;
    let mut confirm = String::new();
    print!("These are all the tasks that will be deployed. Please confirm (y/n): ");
    stdout().flush()?;
    stdin()
        .read_line(&mut confirm)
        .map_err(|e| anyhow!("Error on reading confirmation {}", e))?;
    confirm = confirm.trim().to_string();

    if confirm != "y" {
        err!(anyhow!("Aborted, no changes were made"));
    }

    let docker = DockerService::new()?;
    let abs_path = fs::canonicalize(Path::new(&context))?;
    let built_app_names = new_build_images(deploys.clone(), abs_path, docker.clone()).await?;
    upload_images(docker, built_app_names, user.remote_token.clone()).await?;

    sleep(std::time::Duration::from_millis(1000)).await;
    println!("Deploying...");
    stdout().flush()?;
    API::new(&user.remote_url)?
        .deploy_plan(deploys, user.remote_token)
        .await?;
    println!("Deployed!\n");
    stdout().flush()?;
    ok!(())
}
