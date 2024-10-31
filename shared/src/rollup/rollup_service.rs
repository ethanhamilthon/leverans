use anyhow::anyhow;
use std::collections::HashMap;

use crate::{
    docker::service::{ServiceMount, ServiceParam},
    ok, SecretValue,
};

use super::{
    rollupables::{Rollupable, RollupableService},
    Rollup,
};
use anyhow::Result;

impl Rollup {
    pub async fn rollup_service(
        &self,
        ra: RollupableService,
        ras: &[Rollupable],
        secrets: &[SecretValue],
    ) -> Result<()> {
        // pull image if not exists
        if !self
            .docker
            .list_images()
            .await?
            .iter()
            .any(|image| image.tag == ra.image)
        {
            println!("Pulling image: {}", ra.image);
            self.docker.pull_image(&ra.image).await?;
        }

        // param builder
        let mut docker_params =
            ServiceParam::new(ra.host.clone(), ra.image.clone(), self.network_name.clone());
        self.add_volumes(&mut docker_params, ra.volumes.clone())
            .await?;
        self.add_envs(&mut docker_params, ra.envs.clone(), &ras, secrets);

        // add traefik routing rules
        self.add_routing_rules(
            ra.host,
            ra.domain,
            ra.path_prefix,
            ra.port,
            &mut docker_params,
        )?;
        self.create_or_update_service(docker_params).await
    }

    pub async fn add_volumes(
        &self,
        params: &mut ServiceParam,
        volumes: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let existing_volumes = self
            .docker
            .list_volumes()
            .await?
            .volumes
            .ok_or(anyhow!(""))?;

        if let Some(volumes) = volumes {
            for (key_name, volume_path) in volumes {
                if !existing_volumes.iter().any(|v| v.name == key_name) {
                    self.docker
                        .create_volume(key_name.clone(), volume_path.as_str())
                        .await?;
                }
                params.add_mount(ServiceMount::Volume(key_name, volume_path));
            }
        }

        ok!(())
    }
}
