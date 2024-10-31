pub mod connects;
pub mod db_params;

use anyhow::{anyhow, Result};
use bollard::service;
use db_params::DBParams;
use std::collections::HashMap;

use crate::{
    config::{AppConfig, DbConfig, MainConfig, ServiceConfig},
    err, ok,
};

#[derive(Clone, Debug)]
pub struct RollupableApp {
    pub name: String,
    pub image_suffix: String,
    pub host: String,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub envs: Option<HashMap<String, EnvValues>>,
}

#[derive(Clone, Debug)]
pub struct RollupableDatabase {
    pub name: String,
    pub from: String,
    pub params: DBParams,
    pub depends_on: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct RollupableService {
    pub name: String,
    pub image: String,
    pub host: String,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub envs: Option<HashMap<String, EnvValues>>,
    pub volumes: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug)]
pub enum Rollupable {
    App(RollupableApp),
    Database(RollupableDatabase),
    Service(RollupableService),
}

impl Rollupable {
    pub fn new_from_config(cfg: MainConfig) -> Result<Vec<Self>> {
        let mut result = Vec::new();
        if let Some(dbs) = cfg.db.clone() {
            for (name, db_cfg) in dbs {
                result.push(Self::ra_from_dbcfg(name, cfg.project.clone(), db_cfg)?);
            }
        }
        if let Some(apps) = cfg.app.clone() {
            for (name, app_cfg) in apps {
                result.push(Self::ra_from_appcfg(name, cfg.project.clone(), app_cfg)?);
            }
        }
        if let Some(services) = cfg.service.clone() {
            for (name, services_cfg) in services {
                result.push(Self::ra_from_servicecfg(
                    name,
                    cfg.project.clone(),
                    services_cfg,
                )?);
            }
        }
        ok!(result)
    }

    pub fn ra_from_dbcfg(name: String, project_name: String, db_cfg: DbConfig) -> Result<Self> {
        let from = db_cfg.from.clone();

        ok!(Rollupable::Database(RollupableDatabase {
            name: name.clone(),
            from: from.clone(),
            params: DBParams::from_config(db_cfg.clone(), name.clone(), project_name.clone())
                .ok_or(anyhow::anyhow!("Failed to create db params from config"))?,
            depends_on: None,
        }))
    }

    pub fn ra_from_appcfg(name: String, project_name: String, cfg: AppConfig) -> Result<Self> {
        let mut envs: HashMap<String, EnvValues> = HashMap::new();
        if let Some(cfg_envs) = cfg.env {
            for (k, v) in cfg_envs {
                dbg!("new env", &k, &v, EnvValues::parse_env(&v)?);
                envs.insert(k, EnvValues::parse_env(&v)?);
            }
        }
        ok!(Rollupable::App(RollupableApp {
            name: name.clone(),
            image_suffix: format!("{}-{}-image", project_name, name),
            host: format!("{}-{}-service", project_name, name),
            domain: cfg.domain,
            port: cfg.port,
            path_prefix: cfg.path_prefix,
            depends_on: None,
            envs: Some(envs)
        }))
    }

    pub fn ra_from_servicecfg(
        name: String,
        project_name: String,
        cfg: ServiceConfig,
    ) -> Result<Self> {
        let mut envs: HashMap<String, EnvValues> = HashMap::new();
        if let Some(cfg_envs) = cfg.env {
            for (k, v) in cfg_envs {
                dbg!("new env", &k, &v, EnvValues::parse_env(&v)?);
                envs.insert(k, EnvValues::parse_env(&v)?);
            }
        }
        ok!(Rollupable::Service(RollupableService {
            name: name.clone(),
            image: cfg.image.clone(),
            host: format!("{}-{}-service", project_name, name),
            domain: cfg.domain,
            port: cfg.port,
            path_prefix: cfg.path_prefix,
            depends_on: None,
            envs: Some(envs),
            volumes: cfg.volumes
        }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnvValues {
    This { service: String, method: String },
    Text(String),
    Secret(String),
}

impl EnvValues {
    pub fn parse_env<'a>(value: &'a str) -> Result<EnvValues> {
        if value.chars().count() < 4 {
            ok!(EnvValues::Text(value.to_string()))
        }
        let value = value.trim();
        let first_2_char = &value[..2];
        let last_2_char = &value[value.len() - 2..value.len()];
        if first_2_char == "{{" && last_2_char == "}}" {
            let value = &value[2..value.len() - 2].trim();
            if let Some(env_value) = value.strip_prefix("secret.") {
                ok!(EnvValues::Secret(env_value.to_string()))
            }
            if let Some(env_value) = value.strip_prefix("this.") {
                let parts: Vec<&str> = env_value.splitn(2, '.').collect();
                if parts.len() != 2 {
                    err!(anyhow!(
                        "after \"this\" should be two arguments divided with \".\""
                    ))
                }
                ok!(EnvValues::This {
                    service: parts[0].to_string(),
                    method: parts[1].to_string()
                })
            }
            err!(anyhow!("please use \"this\" or \"secret\""))
        }
        ok!(EnvValues::Text(value.to_string()))
    }
}
