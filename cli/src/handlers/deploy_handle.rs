use std::{
    fs,
    io::{stdin, stdout, Write},
    path::Path,
};

use anyhow::{anyhow, Result};
use scopeguard::defer;
use shared::{console::new_loader, docker::DockerService, err, ok};

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
    skip_confirm: bool,
    unfold: bool,
    rollback: bool,
    timeout: Option<u64>,
) -> Result<()> {
    let (user, deploys) = handle_plan(
        filter,
        only,
        file_name,
        context.clone(),
        to_build,
        unfold,
        rollback,
    )
    .await?;
    if !skip_confirm {
        let mut confirm = String::new();
        print!("Please confirm (y/n): ");
        stdout().flush()?;
        stdin()
            .read_line(&mut confirm)
            .map_err(|e| anyhow!("Error on reading confirmation {}", e))?;
        confirm = confirm.trim().to_string();

        if confirm != "y" {
            err!(anyhow!("Aborted, no changes were made"));
        }
    }

    let docker = DockerService::new()?;
    let abs_path = fs::canonicalize(Path::new(&context))?;
    let loader = if rollback {
        let loader = new_loader("rolling back".to_string());
        loader
    } else {
        let built_app_names = new_build_images(deploys.clone(), abs_path, docker.clone()).await?;
        upload_images(docker, built_app_names, user.remote_token.clone()).await?;

        let loader = new_loader("deploying".to_string());
        loader
    };
    defer! {
        loader.finish()
    }
    let api = API::new(&user.remote_url)?;
    api.deploy_plan(
        deploys.clone(),
        user.remote_token.clone(),
        timeout.unwrap_or(120),
    )
    .await?;
    if rollback {
        loader.finish_with_message("rolled back successfully");
    } else {
        loader.finish_with_message("deployed successfully");
    }
    ok!(())
}
