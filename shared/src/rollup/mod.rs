use std::{future::Future, pin::Pin, str::FromStr, sync::Arc};

use crate::{config::MainConfig, docker::DockerService, ok, Secret, SecretValue};
use anyhow::Result;
use rollup_utils::create_traefik_if_not_exists;
use rollupables::Rollupable;

pub mod rollup_app;
pub mod rollup_db;
pub mod rollup_service;
pub mod rollup_utils;
pub mod rollupables;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RollupMode {
    Local,
    LocalBuild,
    Deploy,
}
type AsyncFn =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

pub struct Rollup {
    mode: RollupMode,
    docker: DockerService,
    write: AsyncFn,
    network_name: String,
}

impl Rollup {
    pub fn new<F, Fut>(
        is_local: bool,
        with_build: bool,
        network_name: String,
        write: F,
    ) -> Result<Self>
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let mut mode = RollupMode::Local;
        if !is_local {
            mode = RollupMode::Deploy
        } else if with_build {
            mode = RollupMode::LocalBuild
        }
        ok!(Self {
            mode,
            docker: DockerService::new()?,
            network_name,
            write: Arc::new(move |s| Box::pin(write(s))),
        })
    }

    pub async fn write_log(&self, s: String) -> Result<()> {
        (self.write)(s).await
    }

    pub async fn rollup(&self, ras: Vec<Rollupable>, secrets: Vec<SecretValue>) -> Result<()> {
        match self.mode {
            RollupMode::Local => self.rollup_local(ras).await,
            RollupMode::LocalBuild => self.rollup_local_build(ras).await,
            RollupMode::Deploy => self.rollup_deploy(ras, secrets).await,
        }
    }

    async fn rollup_local(&self, ras: Vec<Rollupable>) -> Result<()> {
        todo!()
    }

    async fn rollup_local_build(&self, ras: Vec<Rollupable>) -> Result<()> {
        todo!()
    }

    async fn rollup_deploy(&self, ras: Vec<Rollupable>, secrets: Vec<SecretValue>) -> Result<()> {
        for rollupable in &ras {
            match rollupable {
                Rollupable::App(ra_app) => self.rollup_app(ra_app.clone(), &ras, &secrets).await?,
                Rollupable::Database(ra_db) => self.rollup_db(ra_db.clone()).await?,
                Rollupable::Service(ra_service) => {
                    self.rollup_service(ra_service.clone(), &ras, &secrets)
                        .await?
                }
            }
        }
        ok!(())
    }
}

#[tokio::test]
async fn rollup_test() {
    let docker = DockerService::new().unwrap();
    create_traefik_if_not_exists(&docker, "aranea-network".to_string())
        .await
        .unwrap();

    let raw_cfg = r#"
    project: myproj
    app:
        flo:
            build: dockerfile
            context: .
            domain: my.localhost
            port: 3000
            path_prefix: /
            env:
                PG_CONNECTION: "{{ this.musql.connection }}"
    db:
        my-pg:
            from: pg
            username: username123
            password: password123
            dbname: name123
        musql:
            from: pg
    "#;

    let config = MainConfig::from_str(raw_cfg).unwrap();
    let ras = Rollupable::new_from_config(config).unwrap();
    let rollup = Rollup::new(false, false, "aranea-network".to_string(), |s| {
        println!("{}", s);
        Box::pin(async { Ok(()) })
    })
    .unwrap();
    rollup.rollup(ras).await.unwrap();
}
