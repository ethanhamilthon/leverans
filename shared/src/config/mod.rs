use std::{
    collections::HashMap,
    error::Error,
    fmt::{self},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod shared;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainConfig {
    pub project: String,
    pub app: Option<HashMap<String, AppConfig>>,
    pub db: Option<HashMap<String, DbConfig>>,
    pub service: Option<HashMap<String, ServiceConfig>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub build: String,
    pub dockerfile: Option<String>,
    pub context: Option<String>,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConfig {
    pub from: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub dbname: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
}

// Реализуем трейт Display вручную
impl fmt::Display for DbConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "db {}: {}, {}, {}.",
            self.from.clone(),
            self.username.clone().unwrap_or(format!("no data")),
            self.password.clone().unwrap_or(format!("no data")),
            self.dbname.clone().unwrap_or(format!("no data"))
        )
    }
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub image: String,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
}

impl FromStr for MainConfig {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<MainConfig, Box<dyn Error>> {
        let config: MainConfig = serde_yaml::from_str(s)?;
        Ok(config)
    }
}

impl MainConfig {
    pub fn to_string(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
}

#[test]
fn yaml_test() {
    let yaml_text = "project: project-name";
    let mut cfg = MainConfig::from_str(yaml_text).expect("failed to parse");
    assert_eq!(cfg.project, "project-name");
    cfg.project = String::from("another-name");
    let yaml_text = cfg.to_string();
    assert_eq!("project: another-name\n", yaml_text);
}

#[test]
fn yaml_test_2() {
    let yaml_text = "
    project: my-name

    app:
        first-one:
            dockerfile: Dockerfile
        second-one:
            domain: my-domain
    ";

    let cfg = MainConfig::from_str(yaml_text).expect("failed to parse");
    assert_eq!(cfg.project, "my-name");
    let app = cfg.app.as_ref().expect("app must be parsed");
    let app_build = app.get("first-one").unwrap().clone().dockerfile.unwrap();
    let app_domain = app.get("second-one").unwrap().clone().domain.unwrap();
    assert_eq!(app_build, "Dockerfile");
    assert_eq!(app_domain, "my-domain");
}
