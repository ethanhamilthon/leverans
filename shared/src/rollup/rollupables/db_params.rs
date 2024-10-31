use crate::docker::service::ServiceMount;
use crate::{config::DbConfig, ok};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

static DEFAULT_DB_USERNAME: &str = "myuser";
static DEFAULT_DB_PASSWORD: &str = "mypass";
static DEFAULT_DB_ROOT_PASS: &str = "myrootpass";
static DEFAULT_DB_DATABASE: &str = "mydatabase";

#[derive(Clone, Debug)]
pub enum DBParams {
    Postgres {
        username: (String, String),
        password: (String, String),
        database: (String, String),
        port: u16,
        needs: DBParamsNeeds,
    },
    MySQL {
        username: (String, String),
        root_password: (String, String),
        password: (String, String),
        database: (String, String),
        port: u16,
        needs: DBParamsNeeds,
    },
}

impl DBParams {
    pub fn from_config(cfg: DbConfig, name: String, project_name: String) -> Option<Self> {
        match cfg.from.as_str() {
            "mysql" => Some(Self::MySQL {
                username: (
                    format!("MYSQL_USER"),
                    cfg.username.unwrap_or(DEFAULT_DB_USERNAME.to_string()),
                ),
                root_password: (
                    format!("MYSQL_ROOT_PASSWORD"),
                    cfg.password
                        .clone()
                        .unwrap_or(DEFAULT_DB_ROOT_PASS.to_string()),
                ),
                password: (
                    format!("MYSQL_PASSWORD"),
                    cfg.password.unwrap_or(DEFAULT_DB_PASSWORD.to_string()),
                ),
                database: (
                    format!("MYSQL_DATABASE"),
                    cfg.dbname.unwrap_or(DEFAULT_DB_DATABASE.to_string()),
                ),
                port: 3306,
                needs: DBParamsNeeds::default()
                    .add_image_name("mysql:latest".to_string())
                    .add_host(format!("{}-{}-service", project_name, name))
                    .add_volume_name(format!("{}-{}-volume", project_name, name))
                    .add_mounts(vec![ServiceMount::Volume(
                        format!("{}-{}-volume", project_name, name),
                        "/var/lib/mysql".to_string(),
                    )]),
            }),
            "pg" | "postgres" => Some(Self::Postgres {
                username: (
                    format!("POSTGRES_USER"),
                    cfg.username.unwrap_or(DEFAULT_DB_USERNAME.to_string()),
                ),
                password: (
                    format!("POSTGRES_PASSWORD"),
                    cfg.password.unwrap_or(DEFAULT_DB_PASSWORD.to_string()),
                ),
                database: (
                    format!("POSTGRES_DB"),
                    cfg.dbname.unwrap_or("mydatabase".to_string()),
                ),
                port: 5432,
                needs: DBParamsNeeds::default()
                    .add_image_name("postgres:latest".to_string())
                    .add_host(format!("{}-{}-service", project_name, name))
                    .add_volume_name(format!("{}-{}-volume", project_name, name))
                    .add_mounts(vec![ServiceMount::Volume(
                        format!("{}-{}-volume", project_name, name),
                        "/var/lib/postgresql/data".to_string(),
                    )]),
            }),
            _ => None,
        }
    }

    pub fn get_connection(&self) -> Result<String> {
        match self {
            Self::Postgres {
                needs,
                username,
                password,
                database,
                port,
            } => ok!(format!(
                "postgresql://{}:{}@{}:{}/{}",
                username.1,
                password.1,
                needs.host.clone().unwrap_or("localhost".to_string()),
                port,
                database.1
            )),
            Self::MySQL {
                needs,
                username,
                password,
                database,
                port,
                ..
            } => ok!(format!(
                "mysql://{}:{}@{}:{}/{}",
                username.1,
                password.1,
                needs.host.clone().unwrap_or("localhost".to_string()),
                port,
                database.1
            )),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct DBParamsNeeds {
    pub image_name: Option<String>,
    pub host: Option<String>,
    pub volume_name: String,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub mounts: Option<Vec<ServiceMount>>,
}

impl DBParamsNeeds {
    pub fn add_image_name(self, image_name: String) -> Self {
        Self {
            image_name: Some(image_name),
            ..self
        }
    }

    pub fn add_host(self, host: String) -> Self {
        Self {
            host: Some(host),
            ..self
        }
    }

    pub fn add_volume_name(self, volume_name: String) -> Self {
        Self {
            volume_name,
            ..self
        }
    }

    pub fn add_envs(self, envs: HashMap<String, String>) -> Self {
        Self {
            envs: Some(envs),
            ..self
        }
    }

    pub fn add_args(self, args: Vec<String>) -> Self {
        Self {
            args: Some(args),
            ..self
        }
    }

    pub fn add_mounts(self, mounts: Vec<ServiceMount>) -> Self {
        Self {
            mounts: Some(mounts),
            ..self
        }
    }
}
