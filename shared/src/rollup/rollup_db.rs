use crate::{docker::service::ServiceParam, err, ok, rollup::RollupMode};

use super::{
    rollupables::{
        db_params::{DBParams, DBParamsNeeds},
        RollupableDatabase,
    },
    Rollup,
};
use anyhow::{anyhow, Result};

impl Rollup {
    pub async fn rollup_db(&self, ra_db: RollupableDatabase) -> Result<()> {
        println!("Rolling up db: {:?}", ra_db);
        if let DBParams::SQLite { needs } = &ra_db.params {
            self.rollup_db_sqlite(needs.clone()).await?;
            ok!(())
        }
        let needs = match ra_db.params.clone() {
            DBParams::Postgres { needs, .. } => needs,
            DBParams::MySQL { needs, .. } => needs,
            _ => err!(anyhow!("SQLite must be already handled")),
        };
        let mut docker_params = ServiceParam::new(
            needs.host.clone().ok_or(anyhow!("No host name"))?,
            needs.image_name.clone().ok_or(anyhow!("No image name"))?,
            self.network_name.clone(),
        );

        if let Some(mounts) = needs.mounts.clone() {
            mounts.into_iter().for_each(|mount| {
                docker_params.add_mount(mount);
            })
        }

        if let Some(args) = needs.args.clone() {
            docker_params.add_args(args);
        }

        if let Some(envs) = needs.envs.clone() {
            envs.into_iter().for_each(|(key, value)| {
                docker_params.add_env(key, value);
            });
        }

        match ra_db.params {
            DBParams::Postgres {
                username,
                port,
                password,
                database,
                ..
            } => {
                docker_params.add_env(username.0, username.1);
                docker_params.add_env(password.0, password.1);
                docker_params.add_env(database.0, database.1);
                if self.mode == RollupMode::Local {
                    docker_params.add_port(port, port);
                }
            }
            DBParams::MySQL {
                username,
                root_password,
                password,
                database,
                port,
                ..
            } => {
                docker_params.add_env(username.0, username.1);
                docker_params.add_env(password.0, password.1);
                docker_params.add_env(root_password.0, root_password.1);
                docker_params.add_env(database.0, database.1);
                if self.mode == RollupMode::Local {
                    docker_params.add_port(port, port);
                }
            }
            _ => err!(anyhow!("SQLite must be already handled")),
        }

        let exists = self
            .docker
            .is_service_exists(docker_params.get_service_name())
            .await;

        if exists {
            self.docker
                .update_service(docker_params)
                .await
                .map_err(|e| anyhow!(format!("failed to update docker service: {}", e)))?;
        } else {
            self.docker
                .create_volume(needs.volume_name, "local")
                .await
                .map_err(|_| anyhow!(format!("failed to create docker volume")))?;
            self.docker
                .create_service(docker_params)
                .await
                .map_err(|_| anyhow!(format!("failed to create docker service")))?;
        }
        ok!(())
    }

    async fn rollup_db_sqlite(&self, params: DBParamsNeeds) -> Result<()> {
        let exists = self
            .docker
            .list_volumes()
            .await?
            .volumes
            .unwrap()
            .into_iter()
            .any(|volume| volume.name == params.volume_name);
        if !exists {
            self.docker
                .create_volume(params.volume_name.clone(), "local")
                .await?;
            self.docker
                .create_file_for_volume(params.volume_name.clone(), "main.db".to_string())
                .await?;
            ok!(())
        }
        ok!(())
    }
}
