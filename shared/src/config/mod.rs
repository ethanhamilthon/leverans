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
#[serde(deny_unknown_fields)]
pub struct MainConfig {
    pub project: String,
    pub apps: Option<HashMap<String, AppConfig>>,
    pub databases: Option<HashMap<String, DbConfig>>,
    pub services: Option<HashMap<String, ServiceConfig>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub build: Option<String>,
    pub dockerfile: Option<String>,
    pub context: Option<String>,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub cmds: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
    pub proxy: Option<Vec<ConfigProxy>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DbConfig {
    pub from: String,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
    pub backup: Option<BackupParams>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct BackupParams {}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    pub image: String,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub path_prefix: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub cmds: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, String>>,
    pub mounts: Option<HashMap<String, String>>,
    pub proxy: Option<Vec<ConfigProxy>>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigProxy {
    pub domain: String,
    pub port: u16,
    pub path_prefix: Option<String>,
}

impl FromStr for MainConfig {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<MainConfig, Box<dyn Error>> {
        let mut config: MainConfig = serde_yaml::from_str(s)?;
        config.apps = config.apps.map(|a| {
            a.into_iter()
                .map(|b| {
                    let mut c = b.1.clone();
                    c.build = if c.build.is_none() {
                        Some("auto".to_string())
                    } else {
                        c.build.map(|s| {
                            if &s == "auto" || &s == "manual" {
                                s
                            } else {
                                "auto".to_string()
                            }
                        })
                    };
                    (b.0, c)
                })
                .collect::<HashMap<String, AppConfig>>()
        });
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
    let app = cfg.apps.as_ref().expect("app must be parsed");
    let app_build = app.get("first-one").unwrap().clone().dockerfile.unwrap();
    let app_domain = app.get("second-one").unwrap().clone().domain.unwrap();
    assert_eq!(app_build, "Dockerfile");
    assert_eq!(app_domain, "my-domain");
}
