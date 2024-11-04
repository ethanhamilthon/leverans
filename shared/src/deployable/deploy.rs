use std::str::FromStr;

use crate::{config::MainConfig, docker::DockerService, ok, SecretValue};

use super::{Connectable, Deployable};
use anyhow::{anyhow, Result};

pub struct Deploy {
    pub main_config: String,
    pub last_config: Option<String>,
    pub secrets: Vec<SecretValue>,
    pub docker: DockerService,
    pub is_local: bool,
    pub network_name: String,
}

impl Deploy {
    pub async fn deploy(&self) -> Result<()> {
        let all_images: Vec<String> = self
            .docker
            .list_images()
            .await?
            .into_iter()
            .map(|e| e.tag)
            .collect();
        let main_deployables =
            self.config_to_deployable("main", self.config_to_connectable("main")?, &all_images)?;
        let last_deployables =
            self.config_to_deployable("last", self.config_to_connectable("last")?, &all_images)?;
        let deployables = Self::updated_deployables(main_deployables, last_deployables);
        let service_names: Vec<String> = self
            .docker
            .list_services()
            .await?
            .into_iter()
            .map(|s| s.spec.unwrap().name.unwrap())
            .collect();
        for deployable in deployables {
            deployable
                .deploy(
                    self.docker.clone(),
                    service_names.clone(),
                    self.network_name.clone(),
                )
                .await?;
        }
        Ok(())
    }

    pub fn config_to_deployable(
        &self,
        version: &str,
        connectables: Vec<Connectable>,
        images: &[String],
    ) -> Result<Vec<Deployable>> {
        if version == "last" && self.last_config.is_none() {
            return Ok(vec![]);
        }
        let config = match version {
            "last" => {
                let config = MainConfig::from_str(&self.last_config.clone().unwrap())
                    .map_err(|_| anyhow!("cannot parse last config"))?;
                config
            }
            "main" => {
                let config = MainConfig::from_str(&self.main_config)
                    .map_err(|_| anyhow!("cannot parse last config"))?;
                config
            }
            _ => return Err(anyhow!("unknown version {}", version)),
        };
        let mut deployables = vec![];
        if let Some(apps) = config.app {
            for (app_name, app) in apps {
                deployables.push(Deployable::from_app_config(
                    app_name,
                    app,
                    config.project.clone(),
                    images.to_vec(),
                    self.secrets.clone(),
                    connectables.to_vec(),
                )?);
            }
        }

        if let Some(dbs) = config.db {
            for (db_name, db) in dbs {
                deployables.push(Deployable::from_db_config(
                    db_name,
                    db,
                    config.project.clone(),
                    self.secrets.clone(),
                    connectables.to_vec(),
                )?);
            }
        }

        if let Some(services) = config.service {
            for (service_name, service) in services {
                deployables.push(Deployable::from_service_config(
                    service_name,
                    service,
                    config.project.clone(),
                    self.secrets.clone(),
                    connectables.to_vec(),
                )?);
            }
        }
        ok!(deployables)
    }

    pub fn config_to_connectable(&self, version: &str) -> Result<Vec<Connectable>> {
        if version == "last" && self.last_config.is_none() {
            return Ok(vec![]);
        }
        let config = match version {
            "last" => {
                let config = MainConfig::from_str(&self.last_config.clone().unwrap())
                    .map_err(|_| anyhow!("cannot parse last config"))?;
                config
            }
            "main" => {
                let config = MainConfig::from_str(&self.main_config)
                    .map_err(|_| anyhow!("cannot parse last config"))?;
                config
            }
            _ => return Err(anyhow!("unknown version {}", version)),
        };

        let mut connectables = vec![];
        if let Some(apps) = config.app {
            for (app_name, app) in apps {
                connectables.push(Connectable::from_app_config(
                    app_name,
                    app,
                    config.project.clone(),
                )?);
            }
        }

        if let Some(dbs) = config.db {
            for (db_name, db) in dbs {
                connectables.push(Connectable::from_db_config(
                    db_name,
                    db,
                    config.project.clone(),
                )?);
            }
        }

        if let Some(services) = config.service {
            for (service_name, service) in services {
                connectables.push(Connectable::from_service_config(
                    service_name,
                    service,
                    config.project.clone(),
                )?);
            }
        }
        ok!(connectables)
    }

    pub fn updated_deployables(main: Vec<Deployable>, last: Vec<Deployable>) -> Vec<Deployable> {
        let mut deployables = vec![];
        for maind in main {
            if last.contains(&maind) {
                deployables.push(maind);
            }
        }
        deployables
    }
}
