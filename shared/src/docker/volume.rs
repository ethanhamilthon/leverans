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

    pub async fn create_file_for_volume(&self, name: String, file_name: String) -> Result<()> {
        let create_command = format!("touch /data/{}", file_name.clone());
        let alpine_exists = self
            .list_images()
            .await?
            .into_iter()
            .any(|image| image.tag == "alpine" || image.tag == "alpine:latest");
        if !alpine_exists {
            self.pull_image("alpine:latest").await?
        }
        let exists = self
            .list_volumes()
            .await?
            .volumes
            .unwrap()
            .into_iter()
            .any(|volume| volume.name == name);
        if !exists {
            err!(anyhow!(format!("volume not found")))
        }
        let container_options = CreateContainerOptions {
            name: "sqlite-container",
            platform: None,
        };

        let container_config = Config {
            image: Some("alpine"),
            volumes: Some(HashMap::from([(name.as_str(), HashMap::new())])),
            host_config: Some(HostConfig {
                mounts: Some(vec![bollard::models::Mount {
                    typ: Some(MountTypeEnum::VOLUME),
                    target: Some("/data".to_string()),
                    source: Some(name.clone()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            cmd: Some(vec!["sh", "-c", &create_command]),
            ..Default::default()
        };
        self.conn
            .create_container(Some(container_options), container_config)
            .await?;

        // Запускаем контейнер
        self.conn
            .start_container::<String>("sqlite-container", None)
            .await?;

        // Ожидаем завершения работы контейнера (например, через пару секунд)
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Останавливаем контейнер
        self.conn.stop_container("sqlite-container", None).await?;

        // Удаляем контейнер после завершения работы
        self.conn.remove_container("sqlite-container", None).await?;
        ok!(())
    }
}
