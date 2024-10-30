use anyhow::{anyhow, Result};
use std::str::FromStr;

use crate::config::{shared::EnvValueType, MainConfig};
use db_params::{get_db_conn_str, get_db_params, GetDBConnParams};

use crate::{
    docker::{
        image::DockerImage,
        service::{ServiceMount, ServiceParam},
        DockerService,
    },
    err, ok, NETWORK_NAME,
};

pub mod db_params;

pub struct ConfigProcess {
    pub docker_conn: DockerService,
    pub with_local: bool,
}

#[derive(Clone, Debug)]
pub struct DbConnStr {
    pub service_name: String,
    pub conn_str: String,
    pub mount_params: Option<(String, String)>,
}

impl ConfigProcess {
    pub fn new(with_local: bool) -> Self {
        Self {
            docker_conn: DockerService::new().unwrap(),
            with_local,
        }
    }

    pub async fn deploy_config(&self, config: String) -> Result<()> {
        let cfg = MainConfig::from_str(&config).map_err(|_| anyhow!("failed to parse config"))?;

        // run dbs
        let conn_strs = self.deploy_dbs(&cfg).await?;
        if !self.with_local {
            self.deploy_apps(conn_strs, &cfg).await?;
        }
        ok!(())
    }

    async fn deploy_dbs(&self, config: &MainConfig) -> Result<Vec<DbConnStr>> {
        if config.db.is_none() || config.db.clone().unwrap().is_empty() {
            return Ok(vec![]);
        }
        let mut conn_strs: Vec<DbConnStr> = Vec::new();
        let hm = config.db.clone().unwrap();

        let mut docker_params_list: Vec<(ServiceParam, String)> = vec![];

        for (name, db_cfg) in hm {
            let service_name = format!("{}-{}-service", config.project, name);
            let volume_name = format!("{}-{}-volume", config.project, name);

            if db_cfg.from == "sqlite" && !self.with_local {
                let conn_params = self.handle_sqlite(volume_name, service_name).await?;
                conn_strs.push(conn_params);
                continue;
            }

            let mut db_params = get_db_params(db_cfg.from.clone())?;

            // change params to the params from user if they exist
            db_params.pass = db_params
                .pass
                .map(|(k, v)| (k, db_cfg.password.clone().or(Some(v)).unwrap()));
            db_params.root_pass = db_params
                .root_pass
                .map(|(k, v)| (k, db_cfg.password.or(Some(v)).unwrap()));
            db_params.database = db_params
                .database
                .map(|(k, v)| (k, db_cfg.dbname.or(Some(v)).unwrap()));
            db_params.username = db_params
                .username
                .map(|(k, v)| (k, db_cfg.username.or(Some(v)).unwrap()));

            // Creating params
            let mut docker_params = ServiceParam::new(
                service_name.clone(),
                db_params.image_name.clone(),
                NETWORK_NAME.to_string(),
            );
            db_params
                .ports
                .clone()
                .into_iter()
                .for_each(|port| docker_params.add_port(port, port));
            db_params
                .envs
                .clone()
                .into_iter()
                .for_each(|(k, v)| docker_params.add_env(k, v));

            if let Some((key, value)) = db_params.username.clone() {
                docker_params.add_env(key, value);
            }

            if let Some((key, value)) = db_params.pass.clone() {
                docker_params.add_env(key, value);
            }

            if let Some((key, value)) = db_params.root_pass.clone() {
                docker_params.add_env(key, value);
            }

            if let Some((key, value)) = db_params.database.clone() {
                docker_params.add_env(key, value);
            }

            docker_params.add_mount(ServiceMount::Volume(
                volume_name.clone(),
                db_params.mount_cont,
            ));

            // expose ports if it is local
            if self.with_local {
                docker_params.add_port(
                    db_params.ports.first().unwrap().clone(),
                    db_params.ports.first().unwrap().clone(),
                );
            }

            docker_params_list.push((docker_params, volume_name));

            // push connection string
            let conn_params = GetDBConnParams {
                from: db_cfg.from,
                host: if self.with_local {
                    "localhost".to_string()
                } else {
                    service_name.clone()
                },
                username: db_params.username.map(|(_, value)| value),
                pass: db_params.pass.map(|(_, value)| value),
                root_pass: db_params.root_pass.map(|(_, value)| value),
                database: db_params.database.map(|(_, value)| value),
                port: db_params.ports.first().unwrap_or(&0).clone(),
            };

            if let Some(conn_str) = get_db_conn_str(conn_params) {
                conn_strs.push(DbConnStr {
                    service_name: service_name.clone(),
                    conn_str,
                    mount_params: None,
                });
            };
        }

        for (docker_params, volume_name) in docker_params_list {
            let exists = self
                .docker_conn
                .is_service_exists(docker_params.get_service_name())
                .await;

            if exists {
                self.docker_conn
                    .update_service(docker_params)
                    .await
                    .map_err(|e| anyhow!(format!("failed to update docker service: {}", e)))?;
            } else {
                self.docker_conn
                    .create_volume(volume_name, "local")
                    .await
                    .map_err(|_| anyhow!(format!("failed to create docker volume")))?;
                self.docker_conn
                    .create_service(docker_params)
                    .await
                    .map_err(|_| anyhow!(format!("failed to create docker service")))?;
            }
        }
        Ok(conn_strs)
    }

    async fn handle_sqlite(&self, volume_name: String, service_name: String) -> Result<DbConnStr> {
        let exists = self
            .docker_conn
            .list_volumes()
            .await?
            .volumes
            .unwrap()
            .into_iter()
            .any(|volume| volume.name == volume_name);
        if exists {
            let conn = DbConnStr {
                service_name,
                conn_str: format!("sqlite:///data/main.db"),
                mount_params: Some((volume_name, "/data/main.db".to_string())),
            };
            ok!(conn)
        } else {
            self.docker_conn
                .create_volume(volume_name.clone(), "local")
                .await?;
            self.docker_conn
                .create_file_for_volume(volume_name.clone(), "main.db".to_string())
                .await?;
            let conn = DbConnStr {
                service_name,
                conn_str: "sqlite:///data/main.db".to_string(),
                mount_params: Some((volume_name, "/data/main.db".to_string())),
            };
            ok!(conn)
        }
    }

    async fn deploy_apps(&self, conn_strs: Vec<DbConnStr>, config: &MainConfig) -> Result<()> {
        if config.app.is_none() || config.app.clone().unwrap().is_empty() {
            ok!(())
        }
        let mut docker_params_list: Vec<ServiceParam> = vec![];
        let apps = config.app.clone().unwrap();
        for (app_name, app_config) in apps {
            let service_name = format!("{}-{}-service", config.project, app_name);
            let image_prefix = format!("{}-{}-image", config.project, app_name);
            let app_images: Vec<DockerImage> = self
                .docker_conn
                .list_images()
                .await?
                .clone()
                .into_iter()
                .filter(|image| image.tag.starts_with(&image_prefix))
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
                continue;
            }

            // param builder
            let mut docker_params = ServiceParam::new(
                service_name.clone(),
                last_image_name.unwrap(),
                NETWORK_NAME.to_string(),
            );

            // add traefik routing rules
            if app_config.domain.is_some() {
                if app_config.port.is_none() {
                    err!(anyhow!("port is required if there is a domainn"))
                }
                docker_params.add_port(
                    app_config.port.clone().unwrap(),
                    app_config.port.clone().unwrap(),
                );
                docker_params.add_label("traefik.enable".into(), "true".into());
                let mut host_params = format!("Host(`{}`)", app_config.domain.clone().unwrap());
                if app_config.path_prefix.is_some()
                    && app_config.path_prefix.clone().unwrap() != "/"
                {
                    host_params.push_str(
                        format!(
                            " && PathPrefix(`{}`)",
                            app_config.path_prefix.clone().unwrap()
                        )
                        .as_str(),
                    );
                }
                docker_params.add_label(
                    format!("traefik.http.routers.{}.rule", service_name.clone()),
                    host_params,
                );
                docker_params.add_label(
                    format!("traefik.http.routers.{}.service", service_name.clone()),
                    service_name.clone(),
                );
                docker_params.add_label(
                    format!(
                        "traefik.http.services.{}.loadbalancer.server.port",
                        service_name.clone()
                    ),
                    app_config.port.unwrap().to_string(),
                );
                docker_params.add_label(
                    format!("traefik.http.routers.{}.tls", service_name.clone()),
                    "true".into(),
                );
                docker_params.add_label(
                    format!("traefik.http.routers.{}.entrypoints", service_name.clone()),
                    "websecure".into(),
                );
            }

            // add envs
            if app_config.env.is_some() {
                for (env_key, env_value) in app_config.clone().env.unwrap() {
                    if let Ok(env) = MainConfig::parse_env(env_value.as_str()) {
                        match env {
                            EnvValueType::This { service, method } => {
                                let env_service_key =
                                    format!("{}-{}-service", config.project.clone(), service);
                                match method {
                                    "connection" | "conn" | _ => {
                                        if let Some(conn_str_obj) = conn_strs
                                            .clone()
                                            .into_iter()
                                            .find(|x| x.service_name == env_service_key)
                                        {
                                            if let Some((mount_key, mount_value)) =
                                                conn_str_obj.mount_params
                                            {
                                                docker_params.add_mount(ServiceMount::Volume(
                                                    mount_key,
                                                    mount_value,
                                                ));
                                            }
                                            docker_params.add_env(env_key, conn_str_obj.conn_str);
                                        }
                                    }
                                }
                            }
                            EnvValueType::Text(value) => {
                                docker_params.add_env(env_key.clone(), value.to_string())
                            }
                            EnvValueType::Secret(value) => {
                                docker_params.add_env(env_key.clone(), value.to_string())
                            }
                        }
                    }
                }
            }

            // add params to list
            docker_params_list.push(docker_params);
        }

        for param in docker_params_list {
            let exists = self
                .docker_conn
                .is_service_exists(param.get_service_name())
                .await;

            if exists {
                self.docker_conn
                    .update_service(param)
                    .await
                    .map_err(|e| anyhow!(format!("failed to update docker service: {}", e)))?;
            } else {
                self.docker_conn
                    .create_service(param)
                    .await
                    .map_err(|_| anyhow!(format!("failed to create docker service")))?;
            }
        }
        ok!(())
    }
}

#[tokio::test]
async fn db_yaml_test() {
    let yaml_data = r#"
    project: myproj
    app:
        flo:
            build: dockerfile
            context: .
            domain: my-next.localhost
            port: 3000
            path_prefix: /
            env:
                PG_CONNECTION: "{{ this.my-pg.connection }}"
    db:
        my-pg:
            from: pg
            username: mypgapps
            password: mypgpass
            dbname: mypgdatabase
        metrics-db:
            from: sqlite
    "#;
    let cp = ConfigProcess::new(false);
    cp.deploy_config(yaml_data.to_string()).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn services_test() {
    let ds = DockerService::new().unwrap();
    assert_eq!(
        ds.is_service_exists(format!("myproj-my-pg-service")).await,
        true
    );
    assert_eq!(
        ds.is_service_exists(format!("myproj-my-pg-services")).await,
        false
    )
}
