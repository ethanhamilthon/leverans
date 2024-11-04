use anyhow::{anyhow, Result};
use std::{collections::HashMap, time::Duration};

use bollard::{
    secret::{
        EndpointPortConfig, EndpointPortConfigPublishModeEnum, EndpointSpec, Limit, Mount,
        MountTypeEnum, NetworkAttachmentConfig, Service, ServiceCreateResponse, ServiceSpec,
        ServiceSpecMode, ServiceSpecModeReplicated, ServiceSpecUpdateConfig, ServiceUpdateResponse,
        TaskSpec, TaskSpecContainerSpec, TaskSpecPlacement, TaskSpecResources,
    },
    service::{InspectServiceOptions, ListServicesOptions, UpdateServiceOptions},
};

use crate::{ok, strf};

use super::DockerService;

impl DockerService {
    pub async fn list_services(&self) -> Result<Vec<Service>> {
        let filters: HashMap<String, Vec<String>> = HashMap::new();
        let opt = Some(ListServicesOptions {
            filters,
            ..Default::default()
        });
        let services = self.conn.list_services(opt).await?;
        ok!(services)
    }

    pub async fn create_service(&self, params: ServiceParam) -> Result<ServiceCreateResponse> {
        ok!(self
            .conn
            .create_service(params.to_docker_params(), None)
            .await?)
    }

    pub async fn update_service(&self, params: ServiceParam) -> Result<ServiceUpdateResponse> {
        let current_version = self
            .conn
            .inspect_service(&params.name, None::<InspectServiceOptions>)
            .await
            .map_err(|e| anyhow!(format!("error get services {}", e)))?
            .version
            .unwrap()
            .index
            .unwrap();

        let opts = UpdateServiceOptions {
            version: current_version,
            ..Default::default()
        };

        let res = self
            .conn
            .update_service(&params.name, params.to_docker_params(), opts, None)
            .await
            .map_err(|e| anyhow!(format!("error update services {}", e)))?;

        ok!(res)
    }

    pub async fn delete_service(&self, name: String) -> Result<()> {
        ok!(self.conn.delete_service(&name).await?)
    }

    pub async fn is_service_exists(&self, name: String) -> bool {
        if let Ok(services) = self.list_services().await {
            return services
                .into_iter()
                .any(|s| s.spec.as_ref().unwrap().name.as_ref().unwrap() == &name);
        }
        false
    }
}

pub struct ServiceParam {
    // main params
    pub name: String,
    pub image: String,
    pub network_name: String,

    // container params
    pub labels: HashMap<String, String>,
    pub exposed_ports: HashMap<u16, u16>,
    pub envs: HashMap<String, String>,
    pub mounts: Vec<ServiceMount>,
    pub args: Vec<String>,

    // swarm params
    pub cpu: f64,
    pub memory: u32,
    pub replicas: u8,
    pub constraints: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum ServiceMount {
    Volume(String, String),
    Bind(String, String),
}

impl ServiceParam {
    pub fn new(name: String, image: String, network: String) -> ServiceParam {
        ServiceParam {
            name,
            image,
            network_name: network,
            labels: HashMap::new(),
            exposed_ports: HashMap::new(),
            envs: HashMap::new(),
            mounts: vec![],
            args: vec![],
            cpu: 1.0,
            memory: 1024,
            replicas: 1,
            constraints: vec![],
        }
    }

    pub fn get_service_name(&self) -> String {
        self.name.clone()
    }

    pub fn add_label(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
    }

    pub fn add_env(&mut self, key: String, value: String) {
        self.envs.insert(key, value);
    }

    pub fn add_port(&mut self, host: u16, container: u16) {
        self.exposed_ports.insert(host, container);
    }

    pub fn add_mount(&mut self, mount: ServiceMount) {
        self.mounts.push(mount);
    }

    pub fn add_args(&mut self, args: Vec<String>) {
        args.into_iter().for_each(|arg| self.args.push(arg));
    }

    pub fn change_limits(&mut self, cpu: f64, memory: u32) {
        self.cpu = cpu;
        self.memory = memory;
    }

    pub fn set_replicas(&mut self, replicas: u8) {
        self.replicas = replicas;
    }

    pub fn set_constraints(&mut self, constraints: Vec<String>) {
        self.constraints = constraints;
    }

    pub fn to_docker_params(&self) -> ServiceSpec {
        ServiceSpec {
            name: Some(self.name.clone()),
            labels: Some(self.labels.clone()),
            task_template: Some(TaskSpec {
                container_spec: Some(TaskSpecContainerSpec {
                    image: Some(self.image.clone()),
                    labels: Some(self.labels.clone()),
                    args: Some(self.args.clone()),
                    env: Some(
                        self.envs
                            .clone()
                            .into_iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect(),
                    ),
                    mounts: Some(
                        self.mounts
                            .clone()
                            .into_iter()
                            .map(|m| match m {
                                ServiceMount::Volume(f, t) => Mount {
                                    target: Some(t),
                                    source: Some(f),
                                    typ: Some(MountTypeEnum::VOLUME),
                                    ..Default::default()
                                },
                                ServiceMount::Bind(f, t) => Mount {
                                    target: Some(t),
                                    source: Some(f),
                                    typ: Some(MountTypeEnum::BIND),
                                    ..Default::default()
                                },
                            })
                            .collect(),
                    ),
                    ..Default::default()
                }),
                resources: Some(TaskSpecResources {
                    limits: Some(Limit {
                        nano_cpus: Some((self.cpu * 1_000_000_000.0).trunc() as i64),
                        memory_bytes: Some(self.memory as i64 * 1024 * 1024),
                        ..Default::default()
                    }),
                    reservations: None,
                }),
                placement: Some(TaskSpecPlacement {
                    constraints: Some(self.constraints.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            mode: Some(ServiceSpecMode {
                replicated: Some(ServiceSpecModeReplicated {
                    replicas: Some(self.replicas as i64),
                }),
                ..Default::default()
            }),
            update_config: Some(ServiceSpecUpdateConfig {
                parallelism: Some(1),
                delay: Some(10 * 1000 * 1000 * 1000),
                ..Default::default()
            }),
            rollback_config: None,
            networks: Some(vec![NetworkAttachmentConfig {
                target: Some(self.network_name.clone()),
                ..Default::default()
            }]),
            endpoint_spec: Some(EndpointSpec {
                mode: None,
                ports: Some(
                    self.exposed_ports
                        .clone()
                        .into_iter()
                        .map(|(k, v)| EndpointPortConfig {
                            target_port: Some(v as i64),
                            published_port: Some(k as i64),
                            publish_mode: Some(EndpointPortConfigPublishModeEnum::INGRESS),
                            ..Default::default()
                        })
                        .collect(),
                ),
            }),
        }
    }
}

// test to check if it works
#[tokio::test]
#[ignore]
async fn ls_service() {
    let sv = DockerService::new().unwrap();
    let services = sv.list_services().await.unwrap();
    dbg!(&services);
    assert_eq!(services.len(), 1)
}

#[tokio::test]
#[ignore]
async fn service_lifecycle() {
    let ds = DockerService::new().unwrap();
    let mut service_param = ServiceParam::new(
        strf!("nginx-service"),
        strf!("nginx"),
        strf!("aranea-network"),
    );
    service_param.add_port(80, 80);
    let nginx_service = ds.create_service(service_param).await.unwrap();
    assert!(!&nginx_service.id.unwrap().is_empty());

    tokio::time::sleep(Duration::from_secs(10)).await;

    let mut service_update_param = ServiceParam::new(
        strf!("nginx-service"),
        strf!("nginx"),
        strf!("aranea-network"),
    );
    service_update_param.add_port(80, 80);
    service_update_param.add_mount(ServiceMount::Bind(
        strf!("/Users/ethanmotion/pro/stand/nginx.conf"),
        strf!("/etc/nginx/nginx.conf"),
    ));

    ds.update_service(service_update_param).await.unwrap();
}
