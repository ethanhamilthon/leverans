pub mod auth_handle;
pub mod build_handle;
pub mod deploy_handle;
pub mod plan_handle;
pub mod secret_handle;

use std::str::FromStr;

use anyhow::{anyhow, Result};
use shared::{config::MainConfig, docker::DockerService, ok};

use crate::utils::{get_unix_seconds, open_file_as_string};

pub async fn handle_local(with_build: bool) -> Result<()> {
    let docker = DockerService::new()?;
    let yaml_content = open_file_as_string("./deploy.yaml")?;
    let mut yaml_config =
        MainConfig::from_str(&yaml_content).map_err(|_| anyhow!("invalid yaml"))?;
    change_local_domain(&mut yaml_config);

    //if with_build {
    //    if is_traefik_running(&docker).await? {
    //        create_traefik().await?;
    //    }
    //    build_apps(&yaml_config, &docker).await?;
    //    println!("build");
    //} else {
    //    println!("no build");
    //};

    ok!(())
}

pub async fn is_traefik_running(docker: &DockerService) -> Result<bool> {
    let services = docker.list_services().await?;

    ok!(services
        .iter()
        .any(|s| s.spec.clone().unwrap().name.unwrap() == "traefik-service".to_string()))
}

pub async fn create_traefik() -> Result<()> {
    ok!(())
}

pub fn change_local_domain(cfg: &mut MainConfig) {
    if let Some(app) = &mut cfg.app {
        for (_, app_config) in app {
            if let Some(domain) = &mut app_config.domain {
                *domain = domain_to_local(domain);
            }
        }
    }
}

pub fn domain_to_local(domain: &str) -> String {
    let mut parts = domain.split('.').collect::<Vec<&str>>();
    parts = parts[0..parts.len() - 1].to_vec();
    let mut result = parts.join("-");
    result.push_str(".localhost");
    result
}
#[test]
fn config_domain_to_local() {
    let cfg_content = r#"
    project: myapp
    app:
        myapp:
            domain: myapp.example.com
    "#;
    let mut cfg = MainConfig::from_str(cfg_content).unwrap();
    change_local_domain(&mut cfg);
    assert_eq!(
        cfg.app.as_ref().unwrap().get("myapp").unwrap().domain,
        Some("myapp-example.localhost".to_string())
    );
}

#[test]
fn domain_changes() {
    let orig_domain = "example.com";
    let new_domain = domain_to_local(orig_domain);
    assert_eq!(new_domain, "example.localhost");

    let orig_domain = "myapp.example.com";
    let new_domain = domain_to_local(orig_domain);
    assert_eq!(new_domain, "myapp-example.localhost");
}
