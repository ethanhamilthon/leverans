use std::{
    fs,
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use scopeguard::defer;
use shared::{
    config::MainConfig, console::new_loader, docker::DockerService,
    docker_platform::get_docker_platform, err, ok,
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
    to_build: Option<Vec<String>>,
    filter: Option<String>,
    only: Option<Vec<String>>,
) -> Result<()> {
    let (user, deploys) = handle_plan(filter, only, file_name, context.clone(), to_build).await?;
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

    let loader = new_loader("deploying".to_string());
    defer! {
        loader.finish()
    }
    let api = API::new(&user.remote_url)?;
    let mut failed = false;
    let mut err_message = String::new();
    for _ in 0..3 {
        match api
            .deploy_plan(deploys.clone(), user.remote_token.clone())
            .await
        {
            Ok(_) => {
                failed = false;
                break;
            }
            Err(e) => {
                failed = true;
                err_message = e.to_string();
            }
        }
    }
    if failed {
        err!(anyhow!("Failed to deploy: {}", err_message));
    }
    loader.finish_with_message("deployed successfully");
    ok!(())
}
