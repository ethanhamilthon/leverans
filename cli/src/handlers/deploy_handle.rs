use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use shared::{
    config::MainConfig, docker::DockerService, docker_platform::get_docker_platform, err, ok,
};

use crate::{
    api::API,
    data::UserData,
    handlers::build_handle::{build_images, BuildHandle},
    utils::open_file_as_string,
};

pub async fn handle_deploy(
    file_name: String,
    context_path: String,
    no_build: bool,
    filter: Option<String>,
) -> Result<()> {
    if no_build {
        println!("👾 No build Building...: {}", no_build);
        ok!(())
    }
    if filter.is_some() {
        println!("👾 Filtering...: {}", filter.unwrap());
        ok!(())
    }
    let abs_path = fs::canonicalize(Path::new(&context_path))?;
    let config_path = abs_path.join(&file_name);
    let user = UserData::load_db(false).await?.load_current_user().await?;
    let raw_config = open_file_as_string(
        config_path
            .to_str()
            .ok_or(anyhow!("failed to convert path to string"))?,
    )?;
    if !no_build {
        let builder = BuildHandle::new(
            DockerService::new()?,
            abs_path.clone(),
            get_docker_platform().ok(),
        )?;
        build_images(
            builder,
            MainConfig::from_str(&raw_config).map_err(|e| anyhow!("{}", e))?,
            user.remote_token.clone(),
        )
        .await?;
    }
    println!("👾 Deploying...");
    let remote_url = UserData::load_db(false)
        .await?
        .load_current_user()
        .await?
        .remote_url;
    API::new(&remote_url)?
        .upload_config(raw_config, user.remote_token)
        .await?;
    println!("✅ Deployed!\n");
    ok!(())
}
