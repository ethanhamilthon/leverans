pub mod deploy;
pub mod task;

use std::{collections::HashMap, fmt::format, path::PathBuf, u128};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    config::{AppConfig, DbConfig, ServiceConfig},
    docker::{
        service::{ServiceMount, ServiceParam},
        DockerService,
    },
    docker_platform::get_docker_platform,
    err, get_unix_millis, ok, SecretValue, SmartString,
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Deployable {
    // just name of the deployable, without the project name
    pub short_name: String,
    pub project_name: String,
    pub config_type: String,

    // host of the service in docker swarm as well
    pub service_name: String,
    pub docker_image: String,

    pub proxies: Vec<ProxyParams>,

    pub envs: HashMap<String, String>,
    pub volumes: HashMap<String, String>,
    pub mounts: HashMap<String, String>,
    pub args: Vec<String>,
    pub cmd: Vec<String>,

    pub replicas: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ProxyParams {
    pub port: u16,
    pub path_prefix: String,
    pub domain: String,
}

pub fn get_last_image_tag(
    images: Vec<String>,
    project_name: String,
    name: String,
) -> Option<String> {
    let image_name = format!("{}-{}-image", project_name, name);
    let versions = images
        .into_iter()
        .filter(|i| {
            if i.starts_with(&image_name) {
                true
            } else {
                false
            }
        })
        .map(|i| i.split(":").last().unwrap().to_string())
        .collect::<Vec<String>>();
    let mut sorted_versions = versions
        .into_iter()
        .map(|v| v.parse::<u128>())
        .filter(|v| v.is_ok())
        .map(|v| v.unwrap())
        .collect::<Vec<u128>>();
    sorted_versions.sort_by(|a, b| b.cmp(a));

    if sorted_versions.len() > 0 {
        Some(image_name + ":" + &sorted_versions[0].to_string())
    } else {
        None
    }
}

impl Deployable {
    pub fn from_app_config(
        name: String,
        config: AppConfig,
        project_name: String,
        secrets: Vec<SecretValue>,
        connectables: Vec<Connectable>,
        buildables: Vec<Buildable>,
        images: Vec<String>,
    ) -> Result<Self> {
        // find right image
        dbg!(&buildables);
        let this_build = buildables.into_iter().find(|b| b.short_name == name);
        let last_image_name = get_last_image_tag(images, project_name.clone(), name.clone());
        let image_name = if this_build.is_some() {
            this_build.unwrap().tag.clone()
        } else if last_image_name.is_some() {
            last_image_name.unwrap()
        } else {
            err!(anyhow!("could not find image for {}", name))
        };
        // routing
        let mut proxy = if config.port.is_some() && config.domain.is_some() {
            vec![ProxyParams {
                port: config.port.unwrap(),
                path_prefix: config.path_prefix.unwrap_or("/".to_string()),
                domain: config.domain.unwrap(),
            }]
        } else {
            vec![]
        };
        if let Some(proxies) = config.proxy {
            for p in proxies {
                proxy.push(ProxyParams {
                    port: p.port,
                    path_prefix: p.path_prefix.unwrap_or("/".to_string()),
                    domain: p.domain,
                });
            }
        }

        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            config_type: "app".to_string(),
            service_name: get_service_name(&name, &project_name),
            docker_image: image_name,
            proxies: proxy,
            envs: final_envs(config.envs, connectables, secrets),
            volumes: config.volumes.unwrap_or(HashMap::new()),
            mounts: config.mounts.unwrap_or(HashMap::new()),
            args: config.args.unwrap_or(vec![]),
            cmd: vec![],
            replicas: 2,
        })
    }

    pub fn from_service_config(
        name: String,
        config: ServiceConfig,
        project_name: String,
        secrets: Vec<SecretValue>,
        connectables: Vec<Connectable>,
    ) -> Result<Self> {
        let mut proxy = if config.port.is_some() && config.domain.is_some() {
            vec![ProxyParams {
                port: config.port.unwrap(),
                path_prefix: config.path_prefix.unwrap_or("/".to_string()),
                domain: config.domain.unwrap(),
            }]
        } else {
            vec![]
        };
        if let Some(proxies) = config.proxy {
            for p in proxies {
                proxy.push(ProxyParams {
                    port: p.port,
                    path_prefix: p.path_prefix.unwrap_or("/".to_string()),
                    domain: p.domain,
                });
            }
        }

        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            config_type: "service".to_string(),
            service_name: get_service_name(&name, &project_name),
            docker_image: config.image,
            proxies: proxy,
            envs: final_envs(config.envs, connectables, secrets),
            volumes: config.volumes.unwrap_or(HashMap::new()),
            mounts: config.mounts.unwrap_or(HashMap::new()),
            args: config.args.unwrap_or(vec![]),
            cmd: vec![],
            replicas: 1,
        })
    }

    pub fn from_db_config(
        name: String,
        config: DbConfig,
        project_name: String,
        secrets: Vec<SecretValue>,
        connectables: Vec<Connectable>,
    ) -> Result<Self> {
        // Define default settings
        let mut envs;
        let mut volumes = HashMap::new();

        // Override default settings
        let image_name = match config.from.as_str() {
            "postgres" | "pg" | "postgresql" => {
                envs =
                    get_default_envs("postgres").ok_or(anyhow!("No default envs for postgres"))?;
                volumes.insert(
                    get_db_volume_name(&name, &project_name),
                    "/var/lib/postgresql/data".to_string(),
                );
                "postgres".to_string()
            }
            "mysql" => {
                envs = get_default_envs("mysql").ok_or(anyhow!("No default envs for mysql"))?;
                volumes.insert(
                    get_db_volume_name(&name, &project_name),
                    "/var/lib/mysql".to_string(),
                );
                "mysql".to_string()
            }
            typ => {
                err!(anyhow!("Invalid database type, your type is {}", typ))
            }
        };
        let user_envs = final_envs(config.envs, connectables, secrets);
        for (k, v) in user_envs {
            envs.insert(k, v);
        }

        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            config_type: "db".to_string(),
            service_name: get_service_name(&name, &project_name),
            docker_image: image_name,
            proxies: vec![],
            envs,
            volumes: volumes,
            mounts: config.mounts.unwrap_or(HashMap::new()),
            args: config.args.unwrap_or(vec![]),
            cmd: vec![],
            replicas: 1
        })
    }

    pub async fn deploy(
        &self,
        docker: DockerService,
        service_names: Vec<String>,
        network_name: String,
        is_https: bool,
    ) -> Result<()> {
        println!("Deploying {}", self.service_name);
        dbg!(self);
        if service_names.contains(&self.service_name) {
            docker
                .update_service(self.to_docker_params(network_name, is_https)?)
                .await?;
            ok!(())
        } else {
            docker
                .create_service(self.to_docker_params(network_name, is_https)?)
                .await?;
            ok!(())
        }
    }

    pub fn to_docker_params(&self, network_name: String, is_https: bool) -> Result<ServiceParam> {
        let mut service_mounts = vec![];

        for (k, v) in &self.volumes {
            service_mounts.push(ServiceMount::Volume(k.clone(), v.clone()));
        }

        for (k, v) in &self.mounts {
            service_mounts.push(ServiceMount::Bind(k.clone(), v.clone()));
        }
        ok!(ServiceParam {
            name: self.service_name.clone(),
            image: self.docker_image.clone(),
            network_name: network_name,
            labels: self.get_labels(is_https),
            exposed_ports: HashMap::new(),
            envs: self.envs.clone(),
            mounts: service_mounts,
            args: self.args.clone(),
            cpu: 1.0,
            memory: 1024,
            replicas: self.replicas.try_into()?,
            constraints: vec![],
        })
    }

    pub fn get_labels(&self, is_https: bool) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        if self.proxies.is_empty() {
            return labels;
        }
        labels.insert("traefik.enable".into(), "true".into());
        let mut proxy_counter = 0;
        self.proxies.iter().for_each(|p| {
            proxy_counter += 1;
            let domain = &p.domain;
            let path_prefix = &p.path_prefix;
            let host = &format!("{}-{}", self.service_name, proxy_counter);
            let port = &p.port;
            let mut host_params = format!("Host(`{}`)", domain.clone());
            if path_prefix != "/" {
                host_params.push_str(format!(" && PathPrefix(`{}`)", path_prefix.clone()).as_str());
            }
            labels.insert(
                format!("traefik.http.routers.{}.rule", host.clone()),
                host_params,
            );
            labels.insert(
                format!("traefik.http.routers.{}.service", host.clone()),
                host.clone(),
            );
            labels.insert(
                format!(
                    "traefik.http.services.{}.loadbalancer.server.port",
                    host.clone()
                ),
                port.to_string(),
            );
            if is_https {
                labels.insert(
                    format!("traefik.http.routers.{}.tls", host.clone()),
                    "true".into(),
                );
                labels.insert(
                    format!("traefik.http.routers.{}.tls.certresolver", host.clone()),
                    "myresolver".into(),
                );
                labels.insert(
                    format!("traefik.http.routers.{}.entrypoints", host.clone()),
                    "websecure".into(),
                );
            } else {
                labels.insert(
                    format!("traefik.http.routers.{}.entrypoints", host.clone()),
                    "web".into(),
                );
            }
        });
        labels
    }
}

pub fn get_default_envs(service: &str) -> Option<HashMap<String, String>> {
    match service {
        "mysql" => {
            let mut envs = HashMap::new();
            envs.insert("MYSQL_DATABASE".to_string(), "mydb".to_string());
            envs.insert("MYSQL_USER".to_string(), "myuser".to_string());
            envs.insert("MYSQL_PASSWORD".to_string(), "mypassword".to_string());
            envs.insert(
                "MYSQL_ROOT_PASSWORD".to_string(),
                "myrootpassword".to_string(),
            );
            Some(envs)
        }
        "postgres" | "pg" | "postgresql" => {
            let mut envs = HashMap::new();
            envs.insert("POSTGRES_DB".to_string(), "mydb".to_string());
            envs.insert("POSTGRES_USER".to_string(), "mypguser".to_string());
            envs.insert("POSTGRES_PASSWORD".to_string(), "mypassword".to_string());
            Some(envs)
        }
        _ => None,
    }
}

pub fn final_envs(
    envs: Option<HashMap<String, String>>,
    connectables: Vec<Connectable>,
    secrets: Vec<SecretValue>,
) -> HashMap<String, String> {
    if envs.is_none() {
        println!("No envs found");
        return HashMap::new();
    }
    dbg!(&envs);
    let final_envs: HashMap<String, String> = envs
        .unwrap()
        .into_iter()
        .map(|(key, value)| (key, SmartString::parse_env(&value).ok()))
        .map(|(k, v)| {
            let v = v.unwrap_or(SmartString::Text("not found".to_string()));
            let final_value = match v {
                SmartString::This { service, method } => {
                    let connectable = connectables.iter().find(|c| c.short_name == service);
                    match method.as_str() {
                        "connection" | "conn" => {
                            if let Some(connectable) = connectable {
                                connectable.connection.clone().unwrap_or("".to_string())
                            } else {
                                "connectable not found".to_string()
                            }
                        }
                        "internal" | "link" | "url" => {
                            if let Some(connectable) = connectable {
                                connectable.internal_link.clone().unwrap_or("".to_string())
                            } else {
                                "connectable not found".to_string()
                            }
                        }
                        _ => "".to_string(),
                    }
                }
                SmartString::Text(text) => text,
                SmartString::Secret(secret_key) => secrets
                    .iter()
                    .find(|s| s.key == secret_key)
                    .map(|s| s.value.clone())
                    .unwrap_or("".to_string()),
            };

            (k, final_value)
        })
        .collect();

    final_envs
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connectable {
    pub short_name: String,
    pub project_name: String,

    pub connection: Option<String>,
    pub internal_link: Option<String>,
}

impl Connectable {
    pub fn from_service_config(
        name: String,
        config: ServiceConfig,
        project_name: String,
    ) -> Result<Self> {
        let connection = None;
        let mut internal_link = None;
        if config.domain.is_some() && config.port.is_some() {
            internal_link = Some(format!(
                "{}:{}",
                get_service_name(&name, &project_name),
                config.port.unwrap()
            ));
        }
        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            connection,
            internal_link
        })
    }

    pub fn from_db_config(name: String, config: DbConfig, project_name: String) -> Result<Self> {
        let connection = match config.from.as_str() {
            "postgres" | "pg" | "postgresql" => {
                let default_envs =
                    get_default_envs("postgres").ok_or(anyhow!("No default envs for postgres"))?;
                let user_envs = config.envs.unwrap_or(HashMap::new());
                let username = user_envs
                    .get("POSTGRES_USER")
                    .unwrap_or(&default_envs["POSTGRES_USER"]);
                let password = user_envs
                    .get("POSTGRES_PASSWORD")
                    .unwrap_or(&default_envs["POSTGRES_PASSWORD"]);
                let dbname = user_envs
                    .get("POSTGRES_DB")
                    .unwrap_or(&default_envs["POSTGRES_DB"]);
                Some(format!(
                    "postgres://{}:{}@{}:5432/{}",
                    username,
                    password,
                    get_service_name(&name, &project_name),
                    dbname
                ))
            }

            "mysql" => {
                let default_envs =
                    get_default_envs("mysql").ok_or(anyhow!("No default envs for mysql"))?;
                let user_envs = config.envs.unwrap_or(HashMap::new());
                let username = user_envs
                    .get("MYSQL_USER")
                    .unwrap_or(&default_envs["MYSQL_USER"]);
                let password = user_envs
                    .get("MYSQL_PASSWORD")
                    .unwrap_or(&default_envs["MYSQL_PASSWORD"]);
                let dbname = user_envs
                    .get("MYSQL_DATABASE")
                    .unwrap_or(&default_envs["MYSQL_DATABASE"]);
                Some(format!(
                    "mysql://{}:{}@{}:3306/{}",
                    username,
                    password,
                    get_service_name(&name, &project_name),
                    dbname
                ))
            }
            typ => {
                err!(anyhow!("Invalid database type, your type is {}", typ))
            }
        };

        let internal_link = None;
        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            connection,
            internal_link
        })
    }

    pub fn from_app_config(name: String, config: AppConfig, project_name: String) -> Result<Self> {
        let connection = None;
        let mut internal_link = None;
        if config.domain.is_some() && config.port.is_some() {
            internal_link = Some(format!(
                "{}:{}",
                get_service_name(&name, &project_name),
                config.port.unwrap()
            ));
        }
        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            connection,
            internal_link
        })
    }
}

pub fn get_service_name(name: &str, project_name: &str) -> String {
    format!("{}-{}-service", project_name, name)
}

pub fn get_db_volume_name(name: &str, project_name: &str) -> String {
    format!("{}-{}-volume", project_name, name)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Buildable {
    pub short_name: String,
    pub project_name: String,
    pub docker_file_name: String,
    pub context: PathBuf,
    pub tag: String,
    pub platform: String,
}

impl Buildable {
    pub fn from_app_config(name: String, config: AppConfig, project_name: String) -> Result<Self> {
        let short_name = name.clone();
        let project_name = project_name.clone();

        let docker_file_name = config.dockerfile.unwrap_or("Dockerfile".to_string());
        let context = PathBuf::from(config.context.unwrap_or(".".to_string()));

        let tag = format!("{}-{}-image:{}", project_name, name, get_unix_millis());
        let platform = get_docker_platform()?;

        ok!(Self {
            short_name,
            project_name,
            docker_file_name,
            context,
            tag,
            platform
        })
    }
}

// pub struct Backupable {
//     pub volumes: Vec<String>,
//     pub service_name: String,
//     pub short_name: String,
//     pub project_name: String,
//     pub s3_access_key: String,
//     pub s3_secret_key: String,
//     pub s3_bucket: String,
//     pub s3_region: Option<String>,
//     pub s3_endpoint: String,
//     pub s3_file_name: String,
//     pub every: Option<u16>,
// }
//
// impl Backupable {
//     pub fn from_db_config(
//         name: String,
//         config: DbConfig,
//         project_name: String,
//         secrets: Vec<SecretValue>,
//         connectables: Vec<Connectable>,
//     ) -> Result<Self> {
//         ok!(Self {
//             volumes: todo!(),
//             service_name: todo!(),
//             short_name: todo!(),
//             project_name,
//             s3_access_key: todo!(),
//             s3_secret_key: todo!(),
//             s3_bucket: todo!(),
//             s3_region: todo!(),
//             s3_endpoint: todo!(),
//             s3_file_name: todo!(),
//             every: todo!(),
//         })
//     }
// }
