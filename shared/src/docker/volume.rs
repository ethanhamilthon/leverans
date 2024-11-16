use std::collections::HashMap;

use crate::{err, ok};

use super::DockerService;
use anyhow::{anyhow, Result};
use bollard::{
    container::{Config, CreateContainerOptions},
    secret::{HostConfig, MountTypeEnum, VolumeListResponse},
    volume::CreateVolumeOptions,
};

impl DockerService {
    pub async fn list_volumes(&self) -> Result<VolumeListResponse> {
        self.conn
            .list_volumes::<String>(None)
            .await
            .map_err(|_| anyhow!(format!("error list docker volumes")))
    }

    pub async fn is_volume_exists(&self, name: String) -> bool {
        if let Ok(res) = self.list_volumes().await {
            if let Some(volumes) = res.volumes {
                return volumes.into_iter().any(|vol| vol.name == name);
            }
        }
        false
    }

    pub async fn create_volume(&self, name: String, driver: &str) -> Result<()> {
        let opts = CreateVolumeOptions {
            name,
            driver: driver.to_string(),
            driver_opts: HashMap::new(),
            labels: HashMap::new(),
        };
        self.conn
            .create_volume(opts)
            .await
            .map_err(|_| anyhow!("failed to create volume"))?;
        Ok(())
    }
}
