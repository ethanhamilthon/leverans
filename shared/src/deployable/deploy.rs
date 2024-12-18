use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    config::MainConfig,
    deployable::{get_parsed_config, get_regex_parsed_config},
    docker::DockerService,
    err, ok, SecretValue,
};

use super::{task::run_deploy_task, Buildable, Connectable, Deployable};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub fn config_to_connectable(config: MainConfig) -> Result<Vec<Connectable>> {
    let mut connectables = vec![];
    if let Some(apps) = config.apps {
        for (app_name, app) in apps {
            connectables.push(Connectable::from_app_config(
                app_name,
                app,
                config.project.clone(),
            )?);
        }
    }

    if let Some(services) = config.services {
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
pub fn config_to_deployable(
    config: MainConfig,
    buildables: Vec<Buildable>,
    images: Vec<String>,
) -> Result<Vec<Deployable>> {
    let mut deployables = vec![];
    if let Some(apps) = config.apps {
        for (app_name, app) in apps {
            deployables.push(Deployable::from_app_config(
                app_name,
                app,
                config.project.clone(),
                buildables.clone(),
                images.clone(),
            )?);
        }
    }

    if let Some(services) = config.services {
        for (service_name, service) in services {
            deployables.push(Deployable::from_service_config(
                service_name,
                service,
                config.project.clone(),
            )?);
        }
    }
    ok!(deployables)
}

pub fn config_to_buildables(
    config: MainConfig,
    to_build: Option<Vec<String>>,
    images: Vec<String>,
    filters: Option<Vec<String>>,
) -> Result<Vec<Buildable>> {
    let mut buildables = vec![];
    let to_build_flat = if to_build.is_some() {
        to_build.unwrap()
    } else {
        vec![]
    };
    if let Some(apps) = config.apps {
        for (app_name, app) in apps {
            let image_exists =
                exists_in_image_list(images.clone(), app_name.clone(), config.project.clone());
            if filters.clone().is_some()
                && !filters.clone().unwrap().contains(&app_name)
                && image_exists
            {
                println!(
                    "there is no build task for {}, because it is filtered ",
                    app_name
                );
                continue;
            }
            if app.build.is_some()
                && app.build.clone().unwrap() == "manual"
                && !to_build_flat.contains(&app_name)
                && image_exists
            {
                println!("there is no build task for {} ", app_name);
                continue;
            }
            buildables.push(Buildable::from_app_config(
                app_name,
                app,
                config.project.clone(),
            )?);
        }
    }
    ok!(buildables)
}

pub fn exists_in_image_list(images: Vec<String>, name: String, project_name: String) -> bool {
    let image_prefix = format!("{}-{}-image", project_name, name);
    for image in images {
        if image.starts_with(&image_prefix) {
            println!("{} exists in image list", image_prefix);
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deploy {
    pub deployable: Deployable,
    pub lifecycle: DeployLifecycle,
    pub connectable: Connectable,
    pub before_tasks: Vec<DeployTask>,
    pub after_tasks: Vec<DeployTask>,
    pub client_tasks: Vec<DeployTask>,
    pub action: DeployAction,
    pub network_name: String,
}

impl PartialEq for Deploy {
    fn eq(&self, other: &Self) -> bool {
        self.deployable == other.deployable
            && self.connectable == other.connectable
            && self.client_tasks == other.client_tasks
            && self.lifecycle == other.lifecycle
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeployTask {
    Build(Buildable),
    HealthCheck(HealthCheckable),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckable {
    pub service_name: String,
    pub wait_sec: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeployLifecycle {
    Always,
    Once,
    Cron(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Hash, Eq)]
pub enum DeployAction {
    Update,
    Create,
    Delete,
    Nothing,
}

#[derive(Debug)]
pub struct PlanParamaters {
    pub main_config: String,
    pub last_deploys: Vec<(String, String)>,
    pub secrets: Vec<SecretValue>,
    pub network_name: String,
    pub filter: Option<Vec<String>>,
    pub to_build: Vec<String>,
    pub images: Vec<String>,
}

impl Deploy {
    pub async fn deploy(&self, docker: DockerService, service_names: Vec<String>) -> Result<()> {
        match self.lifecycle {
            DeployLifecycle::Always => match self.action {
                DeployAction::Update => {
                    if service_names.contains(&self.deployable.service_name) {
                        self.update(docker.clone()).await?;
                    } else {
                        self.create(docker.clone()).await?;
                    }
                    for task in self.after_tasks.clone() {
                        run_deploy_task(task, docker.clone()).await?;
                    }
                    Ok(())
                }
                DeployAction::Create => {
                    if service_names.contains(&self.deployable.service_name) {
                        self.update(docker.clone()).await?;
                    } else {
                        self.create(docker.clone()).await?;
                    }
                    for task in self.after_tasks.clone() {
                        run_deploy_task(task, docker.clone()).await?;
                    }
                    Ok(())
                }
                DeployAction::Delete => {
                    if service_names.contains(&self.deployable.service_name) {
                        self.delete(docker).await
                    } else {
                        ok!(())
                    }
                }
                DeployAction::Nothing => ok!(()),
            },
            _ => ok!(()),
        }
    }

    pub async fn update(&self, docker: DockerService) -> Result<()> {
        let docker_params = self
            .deployable
            .to_docker_params(self.network_name.clone(), true)?;
        docker.update_service(docker_params).await?;
        ok!(())
    }

    pub async fn delete(&self, docker: DockerService) -> Result<()> {
        docker
            .delete_service(self.deployable.service_name.clone())
            .await?;
        ok!(())
    }

    pub async fn create(&self, docker: DockerService) -> Result<()> {
        let docker_params = self
            .deployable
            .to_docker_params(self.network_name.clone(), true)?;
        docker.create_service(docker_params).await?;
        ok!(())
    }
}

pub fn plan(mut params: PlanParamaters) -> Result<Vec<Deploy>> {
    let main_config = MainConfig::from_str(&params.main_config)
        .map_err(|e| anyhow!("cannot parse last config: {}", e.to_string()))?;
    if params.filter.is_some() && params.filter.clone().unwrap().len() == 0 {
        params.filter = None;
    }
    let last_deploys = params
        .last_deploys
        .clone()
        .into_iter()
        .find(|(project_name, _)| project_name == &main_config.project)
        .map(|(_, d)| serde_json::from_str::<Vec<Deploy>>(&d).ok())
        .flatten()
        .map(|ds| {
            let new_ds: Vec<Deploy> = ds
                .into_iter()
                .filter(|d| d.action != DeployAction::Delete)
                .map(|mut d| {
                    d.action = DeployAction::Nothing;
                    d
                })
                .collect();
            new_ds
        });

    // get all configuration // all logic is here
    let connectables = config_to_connectable(main_config.clone())?;
    let mconfig = get_regex_parsed_config(
        main_config.to_string().as_str(),
        &connectables,
        &params.secrets,
    )?;
    let buildables = config_to_buildables(
        mconfig.clone(),
        Some(params.to_build.clone()),
        params.images.clone(),
        params.filter.clone(),
    )?;
    dbg!("parsed config: {}", &mconfig);
    let deployables = config_to_deployable(mconfig, buildables.clone(), params.images.clone())?;

    // get this time deploys // without comparing with last one
    let main_deploys: Vec<_> = deployables
        .into_iter()
        .map::<Result<Deploy>, _>(|d| {
            Ok(Deploy {
                deployable: d.clone(),
                lifecycle: DeployLifecycle::Always,
                connectable: connectables
                    .iter()
                    .find(|c| c.short_name == d.short_name)
                    .ok_or(anyhow!("cannot find connectable"))?
                    .clone(),
                before_tasks: vec![],
                after_tasks: vec![DeployTask::HealthCheck(HealthCheckable {
                    service_name: d.service_name.clone(),
                    wait_sec: 5,
                })],
                client_tasks: if let Some(b) = buildables
                    .iter()
                    .find(|b| b.short_name == d.short_name && b.project_name == d.project_name)
                {
                    vec![DeployTask::Build(b.clone())]
                } else {
                    vec![]
                },
                action: DeployAction::Nothing,
                network_name: params.network_name.clone(),
            })
        })
        .collect();

    let mut final_deploys = vec![];
    for deploy in main_deploys {
        let mut deploy = deploy?;

        // update or create only filterred deploys
        if let Some(filter) = params.filter.clone() {
            if filter.contains(&deploy.deployable.short_name) {
                deploy.action = if last_deploys.is_some()
                    && last_deploys
                        .clone()
                        .unwrap()
                        .into_iter()
                        .map(|d| d.deployable.short_name)
                        .collect::<Vec<_>>()
                        .contains(&deploy.deployable.short_name)
                {
                    DeployAction::Update
                } else {
                    DeployAction::Create
                };
                final_deploys.push(deploy);
            } else {
                if last_deploys.is_some() {
                    let this_def_deploy = last_deploys.clone().unwrap().into_iter().find(|d| {
                        d.deployable.short_name == deploy.deployable.short_name
                            && d.deployable.project_name == deploy.deployable.project_name
                    });
                    if this_def_deploy.is_some() {
                        deploy.action = DeployAction::Nothing;
                        final_deploys.push(deploy);
                    }
                }
            }
        } else {
            // if there is not filter, we will check every deploy, if the deploy is changed we
            // will update, if there was no this deploy before we will create it. If nothing
            // has changed we wont do anything. If deploy exists in last version and doest in
            // new version we will delete it
            if let Some(last_deploy) = last_deploys.clone() {
                // does last version had this deploy
                let exists_in_last = last_deploy
                    .clone()
                    .into_iter()
                    .find(|d| {
                        d.deployable.short_name == deploy.deployable.short_name
                            && d.deployable.project_name == deploy.deployable.project_name
                    })
                    .is_some();
                dbg!(&exists_in_last);
                let is_changed = !last_deploy.contains(&deploy);

                deploy.action = if exists_in_last && is_changed {
                    DeployAction::Update
                } else if exists_in_last && !is_changed {
                    DeployAction::Nothing
                } else {
                    DeployAction::Create
                };
                final_deploys.push(deploy);
            } else {
                deploy.action = DeployAction::Create;
                final_deploys.push(deploy);
            }
        }
    }

    // find deploys to delete
    if let Some(last_deploy) = last_deploys {
        let deploys_to_delete = last_deploy
            .into_iter()
            .filter(|d| {
                !final_deploys
                    .iter()
                    .any(|f| f.deployable.short_name == d.deployable.short_name)
            })
            .collect::<Vec<_>>();

        for mut deploy in deploys_to_delete {
            deploy.action = DeployAction::Delete;
            deploy.client_tasks = vec![];
            final_deploys.push(deploy);
        }
    }
    ok!(final_deploys)
}
