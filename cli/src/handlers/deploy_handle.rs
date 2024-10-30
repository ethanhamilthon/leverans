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

pub struct DeployHandle {
    pub config_path: PathBuf,
    pub context_path: PathBuf,
}

impl DeployHandle {
    pub fn new(file_name: String, context_path: String) -> Result<Self> {
        match fs::canonicalize(Path::new(&context_path)) {
            Ok(absolute_path) => {
                let mut config_path = absolute_path.clone();
                config_path.push(&file_name);
                ok!(Self {
                    config_path: config_path,
                    context_path: absolute_path,
                })
            }
            Err(e) => err!(anyhow!("Failed to canonicalize path: {}", e)),
        }
    }

    pub async fn handle(&self) -> Result<()> {
        let user = UserData::load_db(false).await?.load_current_user().await?;
        let raw_config = open_file_as_string(
            self.config_path
                .to_str()
                .ok_or(anyhow!("failed to convert path to string"))?,
        )?;
        let builder = BuildHandle::new(
            DockerService::new()?,
            self.context_path.clone(),
            get_docker_platform().ok(),
        )?;
        build_images(
            &builder,
            MainConfig::from_str(&raw_config).map_err(|e| anyhow!("{}", e))?,
            user.remote_token.clone(),
        )
        .await?;
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
}
