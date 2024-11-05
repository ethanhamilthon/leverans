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
    pub filter: Option<String>,
}

impl Deploy {
    pub async fn deploy(&self) -> Result<String> {
        let all_images: Vec<String> = self
            .docker
            .list_images()
            .await?
            .into_iter()
            .map(|e| e.tag)
            .collect();
        let main_deployables =
            self.config_to_deployable(self.config_to_connectable()?, &all_images)?;
        let mut last_deployables = if self.last_config.is_some() {
            serde_json::from_str::<Vec<Deployable>>(&self.last_config.clone().unwrap())?
        } else {
            vec![]
        };
        let deployables = if self.filter.is_some() {
            let filter = self.filter.clone().unwrap();
            main_deployables
                .iter()
                .filter(|d| d.short_name == filter)
                .map(|d| d.clone())
                .collect::<Vec<_>>()
        } else {
            Self::updated_deployables(main_deployables.clone(), last_deployables.clone())
        };
        let service_names: Vec<String> = self
            .docker
            .list_services()
            .await?
            .into_iter()
            .map(|s| s.spec.unwrap().name.unwrap())
            .collect();
        for deployable in &deployables {
            deployable
                .deploy(
                    self.docker.clone(),
                    service_names.clone(),
                    self.network_name.clone(),
                    !self.is_local,
                )
                .await?;
        }
        if self.filter.is_some() {
            if let Some(th_deployable) = last_deployables
                .iter_mut()
                .find(|d| d.short_name == self.filter.clone().unwrap())
            {
                if deployables.len() == 1 {
                    *th_deployable = deployables[0].clone();
                }
            } else {
                last_deployables.push(deployables[0].clone());
            }

            let res = serde_json::to_string(&last_deployables)?;
            ok!(res)
        } else {
            Ok(serde_json::to_string(&main_deployables)?)
        }
    }

    pub fn config_to_deployable(
        &self,
        connectables: Vec<Connectable>,
        images: &[String],
    ) -> Result<Vec<Deployable>> {
        let config = MainConfig::from_str(&self.main_config)
            .map_err(|_| anyhow!("cannot parse last config"))?;
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

    pub fn config_to_connectable(&self) -> Result<Vec<Connectable>> {
        let config = MainConfig::from_str(&self.main_config)
            .map_err(|_| anyhow!("cannot parse last config"))?;
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
            println!();
            println!("looking for {}", maind.short_name);
            dbg!(&maind);
            println!();
            if !last.contains(&maind) {
                println!("{} will be deployed", maind.short_name);
                deployables.push(maind);
            }
        }
        println!("{} to be deployed", deployables.len());
        deployables
    }
}
