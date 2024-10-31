use std::collections::HashMap;

use crate::{
    docker::{image::DockerImage, service::ServiceParam},
    err, ok,
    rollup::rollupables::{db_params::DBParams, EnvValues},
    SecretValue,
};
use anyhow::{anyhow, Result};

use super::{
    rollupables::{Rollupable, RollupableApp},
    Rollup,
};

impl Rollup {
    pub async fn rollup_app(
        &self,
        ra_app: RollupableApp,
        ra_all: &[Rollupable],
        secrets: &[SecretValue],
    ) -> Result<()> {
        println!("Rolling up app: {:?}", ra_app);
        let app_images: Vec<DockerImage> = self
            .docker
            .list_images()
            .await?
            .clone()
            .into_iter()
            .filter(|image| image.tag.starts_with(&ra_app.image_suffix))
            .collect();
        if app_images.is_empty() {
            err!(anyhow!("there is no image for this app"))
        }

        // find the latest image
        let mut image_longest_time: u64 = 0;
        let mut last_image_name: Option<String> = None;
        for image in app_images {
            let parts: Vec<&str> = image.tag.splitn(2, ":").collect();
            if parts.is_empty() || parts.len() != 2 {
                continue;
            }
            let image_time: u64 = parts[1].parse().unwrap_or(0);
            if image_time > image_longest_time {
                image_longest_time = image_time;
                last_image_name = Some(parts.join(":"));
            }
        }

        if last_image_name.is_none() {
            err!(anyhow!("can't find any image for this app"))
        }

        // param builder
        let mut docker_params = ServiceParam::new(
            ra_app.host.clone(),
            last_image_name.unwrap(),
            self.network_name.clone(),
        );

        // add traefik routing rules
        self.add_routing_rules(
            ra_app.host,
            ra_app.domain,
            ra_app.path_prefix,
            ra_app.port,
            &mut docker_params,
        )?;

        // add envs
        self.add_envs(&mut docker_params, ra_app.envs, ra_all, secrets);

        // create or update service
        self.create_or_update_service(docker_params).await
    }

    pub async fn create_or_update_service(&self, docker_params: ServiceParam) -> Result<()> {
        let exists = self
            .docker
            .is_service_exists(docker_params.get_service_name())
            .await;

        if exists {
            self.docker
                .update_service(docker_params)
                .await
                .map_err(|e| anyhow!(format!("failed to update docker service: {}", e)))?;
        } else {
            self.docker
                .create_service(docker_params)
                .await
                .map_err(|_| anyhow!(format!("failed to create docker service")))?;
        }
        ok!(())
    }

    pub fn add_envs(
        &self,
        docker_params: &mut ServiceParam,
        envs: Option<HashMap<String, EnvValues>>,
        ras: &[Rollupable],
        secrets: &[SecretValue],
    ) {
        if envs.is_some() {
            for (k, v) in envs.clone().unwrap() {
                match v {
                    EnvValues::Text(v) => docker_params.add_env(k, v),
                    EnvValues::Secret(v) => {
                        for secret in secrets {
                            if secret.key == v {
                                docker_params.add_env(k.clone(), secret.value.clone());
                            }
                        }
                    }
                    EnvValues::This { service, method } => {
                        println!("this: {:?} {:?}", service, method);
                        println!("all: {:?}", ras);
                        let ra_service = ras.iter().find(|ra| match ra {
                            Rollupable::App(rapp) => rapp.name == service,
                            Rollupable::Database(rdb) => rdb.name == service,
                            Rollupable::Service(rsv) => rsv.name == service,
                        });
                        if ra_service.is_some() {
                            match method.as_str() {
                                "conn" | "connection" => {
                                    let this_ra_sv = ra_service.unwrap();
                                    if let Ok(conn_str) = this_ra_sv.get_connection() {
                                        docker_params.add_env(k, conn_str);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn add_routing_rules(
        &self,
        host: String,
        domain: Option<String>,
        path_prefix: Option<String>,
        port: Option<u16>,
        docker_params: &mut ServiceParam,
    ) -> Result<()> {
        if domain.is_some() {
            if port.is_none() {
                err!(anyhow!("port is required if there is a domainn"))
            }
            docker_params.add_label("traefik.enable".into(), "true".into());
            let mut host_params = format!("Host(`{}`)", domain.clone().unwrap());
            if path_prefix.is_some() && path_prefix.clone().unwrap() != "/" {
                host_params.push_str(
                    format!(" && PathPrefix(`{}`)", path_prefix.clone().unwrap()).as_str(),
                );
            }
            docker_params.add_label(
                format!("traefik.http.routers.{}.rule", host.clone()),
                host_params,
            );
            docker_params.add_label(
                format!("traefik.http.routers.{}.service", host.clone()),
                host.clone(),
            );
            docker_params.add_label(
                format!(
                    "traefik.http.services.{}.loadbalancer.server.port",
                    host.clone()
                ),
                port.unwrap().to_string(),
            );
            docker_params.add_label(
                format!("traefik.http.routers.{}.tls", host.clone()),
                "true".into(),
            );
            docker_params.add_label(
                format!("traefik.http.routers.{}.entrypoints", host.clone()),
                "websecure".into(),
            );
        }
        ok!(())
    }
}
