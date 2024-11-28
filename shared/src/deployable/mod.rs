pub mod deploy;
pub mod rollback;
pub mod task;

use std::{collections::HashMap, fmt::format, path::PathBuf, str::FromStr, u128};

use anyhow::{anyhow, Result};
use bollard::secret::TaskSpecRestartPolicyConditionEnum;
use deploy::config_to_connectable;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    config::{AppConfig, ConfigProxy, HealthCheck, MainConfig, ServiceConfig},
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
    pub expose: Vec<u16>,

    pub envs: HashMap<String, String>,
    pub volumes: HashMap<String, String>,
    pub mounts: HashMap<String, String>,
    pub args: Vec<String>,
    pub cmd: Option<Vec<String>>,
    pub user_labels: HashMap<String, String>,

    pub replicas: u32,
    pub cpu: f64,
    pub memory: u64,
    pub restart: String,
    pub constraints: Option<Vec<String>>,

    pub https_enabled: bool,
    pub healthcheck: Option<HealthCheck>,
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
            envs: config.envs.unwrap_or(HashMap::new()),
            volumes: config.volumes.unwrap_or(HashMap::new()),
            expose: config.expose.unwrap_or(vec![]),
            mounts: config.mounts.unwrap_or(HashMap::new()),
            args: config.args.unwrap_or(vec![]),
            user_labels: config.labels.unwrap_or(HashMap::new()),
            cmd: config.cmds,
            restart: config.restart.unwrap_or("always".to_string()),
            replicas: config.replicas.unwrap_or(2),
            constraints: config.constraints,
            cpu: config.cpu.unwrap_or(1.0),
            memory: config.memory.unwrap_or(1024) as u64,
            https_enabled: config.https.unwrap_or(true),
            healthcheck: config.health_check,
        })
    }

    pub fn from_service_config(
        name: String,
        config: ServiceConfig,
        project_name: String,
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
            envs: config.envs.unwrap_or(HashMap::new()),
            volumes: config.volumes.unwrap_or(HashMap::new()),
            mounts: config.mounts.unwrap_or(HashMap::new()),
            args: config.args.unwrap_or(vec![]),
            expose: config.expose.unwrap_or(vec![]),
            user_labels: config.labels.unwrap_or(HashMap::new()),
            cmd: config.cmds,
            replicas: config.replicas.unwrap_or(1),
            constraints: config.constraints,
            restart: config.restart.unwrap_or("always".to_string()),
            cpu: config.cpu.unwrap_or(1.0),
            memory: config.memory.unwrap_or(1024) as u64,
            https_enabled: config.https.unwrap_or(true),
            healthcheck: config.health_check,
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
        let mut labels = self.get_labels(is_https);
        for (k, v) in self.user_labels.clone() {
            labels.insert(k, v); // overwrite with user lab
        }
        let restart = match self.restart.as_str() {
            "always" | "any" => TaskSpecRestartPolicyConditionEnum::ANY,
            "none" | "no" => TaskSpecRestartPolicyConditionEnum::NONE,
            "on-failure" | "failure" => TaskSpecRestartPolicyConditionEnum::ON_FAILURE,
            _ => {
                return Err(anyhow!("Restart policy not supported: {}", self.restart));
            }
        };
        ok!(ServiceParam {
            name: self.service_name.clone(),
            image: self.docker_image.clone(),
            network_name: network_name,
            labels: self.get_labels(is_https),
            exposed_ports: self.expose.clone().into_iter().map(|p| (p, p)).collect(),
            envs: self.envs.clone(),
            mounts: service_mounts,
            args: self.args.clone(),
            cmd: self.cmd.clone(),
            cpu: self.cpu.try_into()?,
            memory: self.memory.try_into()?,
            replicas: self.replicas.try_into()?,
            healthcheck: self.healthcheck.clone(),
            constraints: self.constraints.clone().unwrap_or(vec![]),
            restart: restart,
        })
    }

    pub fn get_labels(&self, mut is_https: bool) -> HashMap<String, String> {
        if !self.https_enabled {
            is_https = self.https_enabled;
        }
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

pub fn get_regex_parsed_config(
    config: &str,
    connectables: &[Connectable],
    secrets: &[SecretValue],
) -> Result<MainConfig> {
    let re = Regex::new(r"\$\{([A-Za-z0-9_\-\/\.]+)\}")?;

    let mut replaced_config = String::new();
    let mut last_match_end = 0;
    dbg!(&config);

    for caps in re.captures_iter(config) {
        let mat = caps.get(0).unwrap();
        dbg!(&mat);
        replaced_config.push_str(&config[last_match_end..mat.start()]);

        let key = caps.get(1).unwrap().as_str();
        dbg!(&key);
        let value = smarter_string(key, connectables, secrets)
            .map_err(|_| anyhow!("Failed to resolve key: {}", key))?;
        replaced_config.push_str(&value);

        last_match_end = mat.end();
    }

    replaced_config.push_str(&config[last_match_end..]);

    let rconfig =
        MainConfig::from_str(&replaced_config).map_err(|_| anyhow!("Failed to parse config"))?;
    Ok(rconfig)
}

pub fn smarter_string(
    s: &str,
    connectables: &[Connectable],
    secrets: &[SecretValue],
) -> Result<String> {
    let value = s.trim();
    dbg!(&value);
    if value.starts_with("secret.") {
        let key = value
            .strip_prefix("secret.")
            .ok_or(anyhow!("Invalid secret key"))?;
        let secret = secrets
            .iter()
            .find(|s| s.key == key)
            .ok_or(anyhow!("Secret not found : {}", key))?;
        ok!(secret.value.clone())
    } else if value.starts_with("this.") {
        let key = value
            .strip_prefix("this.")
            .ok_or(anyhow!("Invalid connectable key"))?;
        dbg!(&key);
        let parts = key.splitn(2, '.').collect::<Vec<_>>();
        let connectable = connectables
            .iter()
            .find(|c| c.short_name == parts[0])
            .ok_or(anyhow!("Connect not found : {}", parts[0]))?;
        let final_str = match parts[1] {
            "internal" => connectable
                .internal_link
                .clone()
                .ok_or(anyhow!("internal not found: {}", parts[0]))?,
            "external" => connectable
                .external_link
                .clone()
                .ok_or(anyhow!("external not found: {}", parts[0]))?,
            "host" => connectable.host.clone().ok_or(anyhow!("host not found"))?,
            "port" => connectable
                .port
                .clone()
                .map(|p| p.to_string())
                .ok_or(anyhow!("port not found"))?,
            _ => err!(anyhow!("unknown method: {}", parts[1])),
        };
        ok!(final_str)
    } else {
        err!(anyhow!("Invalid value: {}", value));
    }
}

pub fn get_parsed_config(
    config: &str,
    connectables: &[Connectable],
    secrets: &[SecretValue],
) -> Result<MainConfig> {
    let mut config = MainConfig::from_str(config).map_err(|_| anyhow!("Failed to parse config"))?;
    let env_func = |envs: HashMap<String, String>| -> Result<HashMap<String, String>> {
        envs.into_iter()
            .map(|(key, value)| {
                let smart_key = to_smart_string(&key, &connectables, &secrets)
                    .map_err(|e| anyhow!("Error processing environment key '{}': {}", key, e))?;
                let smart_value =
                    to_smart_string(&value, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing environment value '{}': {}", value, e)
                    })?;
                Ok((smart_key, smart_value))
            })
            .collect::<Result<HashMap<String, String>>>()
    };
    let vec_func = |elems: Vec<String>| -> Result<Vec<String>> {
        elems
            .into_iter()
            .map(|e| {
                to_smart_string(&e, &connectables, &secrets)
                    .map_err(|err| anyhow!("Error processing element '{}': {}", e, err))
            })
            .collect::<Result<Vec<String>>>()
    };

    config.apps = match config.apps {
        Some(apps) => {
            let mut new_apps = HashMap::new();
            for (appname, mut cfg) in apps {
                cfg.dockerfile = match cfg.dockerfile {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing dockerfile for '{}': {}", appname, e)
                    })?),
                    None => None,
                };

                cfg.context = match cfg.context {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing context for '{}': {}", appname, e)
                    })?),
                    None => None,
                };

                cfg.build_args = match cfg.build_args {
                    Some(build_args) => Some(env_func(build_args)?),
                    None => None,
                };

                cfg.nix_cmds = match cfg.nix_cmds {
                    Some(cmds) => Some(vec_func(cmds)?),
                    None => None,
                };

                cfg.envs = match cfg.envs {
                    Some(envs) => Some(env_func(envs)?),
                    None => None,
                };

                cfg.labels = match cfg.labels {
                    Some(labels) => Some(env_func(labels)?),
                    None => None,
                };

                cfg.volumes = match cfg.volumes {
                    Some(volumes) => Some(env_func(volumes)?),
                    None => None,
                };

                cfg.mounts = match cfg.mounts {
                    Some(mounts) => Some(env_func(mounts)?),
                    None => None,
                };

                cfg.domain = match cfg.domain {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing domain for '{}': {}", appname, e)
                    })?),
                    None => None,
                };

                cfg.path_prefix = match cfg.path_prefix {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing path prefix for '{}': {}", appname, e)
                    })?),
                    None => None,
                };

                cfg.args = match cfg.args {
                    Some(args) => Some(vec_func(args)?),
                    None => None,
                };

                cfg.cmds = match cfg.cmds {
                    Some(cmds) => Some(vec_func(cmds)?),
                    None => None,
                };

                cfg.health_check = match cfg.health_check {
                    Some(mut hc) => {
                        Some({
                            if let Some(cmd) = hc.cmd {
                                let mut new_cmd = Vec::new();
                                for c in cmd {
                                    new_cmd.push(to_smart_string(&c, &connectables, &secrets)
                                        .map_err(|e| {
                                            anyhow!("Error processing health check command for '{}': {}", appname, e)
                                        })?);
                                }
                                hc.cmd = Some(new_cmd);
                            }
                            hc
                        })
                    }
                    None => None,
                };

                cfg.proxy = match cfg.proxy {
                    Some(p) => {
                        let mut new_proxies = Vec::new();
                        for mut proxy in p {
                            proxy.domain = to_smart_string(&proxy.domain, &connectables, &secrets)
                                .map_err(|e| {
                                    anyhow!("Error processing domain for '{}': {}", appname, e)
                                })?;
                            proxy.path_prefix = match proxy.path_prefix {
                                Some(path) => {
                                    Some(to_smart_string(&path, &connectables, &secrets).map_err(
                                        |e| {
                                            anyhow!(
                                                "Error processing path prefix for '{}': {}",
                                                appname,
                                                e
                                            )
                                        },
                                    )?)
                                }
                                None => None,
                            };
                            new_proxies.push(proxy);
                        }
                        Some(new_proxies)
                    }
                    None => None,
                };

                new_apps.insert(appname, cfg);
            }
            Some(new_apps)
        }
        None => None,
    };

    config.services = match config.services {
        Some(services) => {
            let mut new_services = HashMap::new();
            for (service_name, mut cfg) in services {
                cfg.image = to_smart_string(&cfg.image, &connectables, &secrets)
                    .map_err(|e| anyhow!("Error processing image for '{}': {}", service_name, e))?;
                cfg.envs = match cfg.envs {
                    Some(envs) => Some(env_func(envs)?),
                    None => None,
                };

                cfg.labels = match cfg.labels {
                    Some(labels) => Some(env_func(labels)?),
                    None => None,
                };

                cfg.volumes = match cfg.volumes {
                    Some(volumes) => Some(env_func(volumes)?),
                    None => None,
                };

                cfg.mounts = match cfg.mounts {
                    Some(mounts) => Some(env_func(mounts)?),
                    None => None,
                };

                cfg.domain = match cfg.domain {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing domain for '{}': {}", service_name, e)
                    })?),
                    None => None,
                };

                cfg.path_prefix = match cfg.path_prefix {
                    Some(d) => Some(to_smart_string(&d, &connectables, &secrets).map_err(|e| {
                        anyhow!("Error processing path prefix for '{}': {}", service_name, e)
                    })?),
                    None => None,
                };

                cfg.args = match cfg.args {
                    Some(args) => Some(vec_func(args)?),
                    None => None,
                };

                cfg.cmds = match cfg.cmds {
                    Some(cmds) => Some(vec_func(cmds)?),
                    None => None,
                };

                cfg.health_check = match cfg.health_check {
                    Some(mut hc) => {
                        Some({
                            if let Some(cmd) = hc.cmd {
                                let mut new_cmd = Vec::new();
                                for c in cmd {
                                    new_cmd.push(to_smart_string(&c, &connectables, &secrets)
                                        .map_err(|e| {
                                            anyhow!("Error processing health check command for '{}': {}", service_name, e)
                                        })?);
                                }
                                hc.cmd = Some(new_cmd);
                            }
                            hc
                        })
                    }
                    None => None,
                };

                cfg.proxy = match cfg.proxy {
                    Some(p) => {
                        let mut new_proxies = Vec::new();
                        for mut proxy in p {
                            proxy.domain = to_smart_string(&proxy.domain, &connectables, &secrets)
                                .map_err(|e| {
                                    anyhow!("Error processing domain for '{}': {}", service_name, e)
                                })?;
                            proxy.path_prefix = match proxy.path_prefix {
                                Some(path) => {
                                    Some(to_smart_string(&path, &connectables, &secrets).map_err(
                                        |e| {
                                            anyhow!(
                                                "Error processing path prefix for '{}': {}",
                                                service_name,
                                                e
                                            )
                                        },
                                    )?)
                                }
                                None => None,
                            };
                            new_proxies.push(proxy);
                        }
                        Some(new_proxies)
                    }
                    None => None,
                };

                new_services.insert(service_name, cfg);
            }
            Some(new_services)
        }
        None => None,
    };
    ok!(config)
}

pub fn to_smart_string(
    value: &str,
    connectables: &[Connectable],
    secrets: &[SecretValue],
) -> Result<String> {
    let parsed = SmartString::parse_env(value)?;
    let mut fin: Vec<String> = vec![];
    for p in parsed {
        let value = match p {
            SmartString::This { service, method } => {
                let connectable = connectables
                    .iter()
                    .find(|c| c.short_name == service)
                    .ok_or(anyhow!("could not connect to {}", service))?;
                let res = match method.as_str() {
                    "internal" => connectable
                        .internal_link
                        .clone()
                        .ok_or(anyhow!("internal not found: {}", service))?,
                    "external" => connectable
                        .external_link
                        .clone()
                        .ok_or(anyhow!("external not found"))?,
                    "host" => connectable.host.clone().ok_or(anyhow!("host not found"))?,
                    "port" => connectable
                        .port
                        .clone()
                        .map(|p| p.to_string())
                        .ok_or(anyhow!("port not found"))?,
                    _ => err!(anyhow!("unknown method: {}", method)),
                };
                res
            }
            SmartString::Text(text) => text,
            SmartString::Secret(secret_key) => secrets
                .iter()
                .find(|s| s.key == secret_key)
                .map(|s| s.value.clone())
                .ok_or(anyhow!("secret not found: {}", secret_key))?,
        };
        fin.push(value);
    }

    Ok(fin.join(""))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Connectable {
    pub short_name: String,
    pub project_name: String,
    pub internal_link: Option<String>,
    pub external_link: Option<String>,

    pub host: Option<String>,
    pub port: Option<u16>,
}

impl Connectable {
    pub fn from_service_config(
        name: String,
        config: ServiceConfig,
        project_name: String,
    ) -> Result<Self> {
        let mut internal_link = None;
        let mut external_link = None;
        if config.port.is_some() {
            internal_link = Some(format!(
                "{}:{}",
                get_service_name(&name, &project_name),
                config.port.unwrap()
            ));
        }
        if config.domain.is_some() && config.port.is_some() {
            external_link = Some(format!("https://{}", config.domain.unwrap()));
        }
        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            internal_link,
            external_link,
            host: Some(get_service_name(&name, &project_name)),
            port: config.port.clone(),
        })
    }

    pub fn from_app_config(name: String, config: AppConfig, project_name: String) -> Result<Self> {
        let mut internal_link = None;
        let mut external_link = None;
        if config.port.is_some() {
            internal_link = Some(format!(
                "{}:{}",
                get_service_name(&name, &project_name),
                config.port.unwrap()
            ));
        }
        if config.domain.is_some() && config.port.is_some() {
            if config.https.is_some() && !config.https.unwrap() {
                external_link = Some(format!("http://{}", config.domain.unwrap()));
            } else {
                external_link = Some(format!("https://{}", config.domain.unwrap()));
            }
        }
        ok!(Self {
            short_name: name.clone(),
            project_name: project_name.clone(),
            internal_link,
            external_link,
            host: Some(get_service_name(&name, &project_name)),
            port: config.port.clone(),
        })
    }
}

pub fn get_service_name(name: &str, project_name: &str) -> String {
    format!("{}-{}-service", project_name, name)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Buildable {
    pub short_name: String,
    pub project_name: String,
    pub is_nix: bool,
    pub nix_cmds: Vec<String>,
    pub docker_file_name: String,
    pub context: PathBuf,
    pub tag: String,
    pub platform: String,
    pub build_args: Option<HashMap<String, String>>,
}

impl Buildable {
    pub fn from_app_config(name: String, config: AppConfig, project_name: String) -> Result<Self> {
        let short_name = name.clone();
        let project_name = project_name.clone();
        let context = PathBuf::from(config.context.unwrap_or(".".to_string()));
        let platform = get_docker_platform()?;
        let tag = format!("{}-{}-image:{}", project_name, name, get_unix_millis());

        if config.builder.is_some()
            && (&config.builder.clone().unwrap() == "nix" || &config.builder.unwrap() == "nixpacks")
        {
            let mut image_name_found = false;
            let cmds = if config.nix_cmds.is_some() {
                let mut raw_cmds = config.nix_cmds.unwrap();
                for cmd in raw_cmds.iter_mut() {
                    if cmd == "<tag>" {
                        *cmd = tag.clone();
                        image_name_found = true;
                    } else if cmd == "<context>" {
                        *cmd = context
                            .to_str()
                            .ok_or(anyhow!("Failed to convert context path to string"))?
                            .to_string();
                    }
                }
                if !image_name_found {
                    return Err(anyhow!("Failed to find image name in nix_cmds"));
                }
                raw_cmds
            } else {
                vec![
                    "nixpacks".to_string(),
                    "build".to_string(),
                    context
                        .to_str()
                        .ok_or(anyhow!("Failed to convert context path to string"))?
                        .to_string(),
                    "--name".to_string(),
                    tag.clone(),
                    "--platform".to_string(),
                    platform.clone(),
                ]
            };
            ok!(Self {
                short_name,
                project_name,
                docker_file_name: "".to_string(),
                context,
                is_nix: true,
                nix_cmds: cmds,
                tag,
                platform,
                build_args: config.build_args,
            })
        } else {
            let docker_file_name = config.dockerfile.unwrap_or("Dockerfile".to_string());

            ok!(Self {
                short_name,
                project_name,
                docker_file_name,
                context,
                is_nix: false,
                nix_cmds: vec![],
                tag,
                platform,
                build_args: config.build_args,
            })
        }
    }
}

#[test]
fn parser_test() {
    let raw_config = r#"
    project: my-pro
    apps: 
        app1:
            domain: ${secret.app1-domain}
            port: 8080
    services:
        s1:
            image: ${secret.s1-image}
            port: 3000
            envs:
                APP1_LINK: ${this.app1.external}
            labels:
                ${secret.s1-label}: ${this.app1.host}
    "#;
    let secrets = vec![
        SecretValue {
            key: "app1-domain".to_string(),
            value: "my-domain.com".to_string(),
        },
        SecretValue {
            key: "s1-image".to_string(),
            value: "postgres:16".to_string(),
        },
        SecretValue {
            key: "s1-label".to_string(),
            value: "app-host".to_string(),
        },
    ];
    let connectables = config_to_connectable(MainConfig::from_str(raw_config).unwrap()).unwrap();
    let parsed_config = get_regex_parsed_config(&raw_config, &connectables, &secrets).unwrap();
    dbg!(parsed_config);
}
