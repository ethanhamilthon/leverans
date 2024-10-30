use anyhow::Result;

use crate::{
    docker::{
        service::{ServiceMount, ServiceParam},
        DockerService,
    },
    ok,
};

pub async fn create_traefik_if_not_exists(
    docker: &DockerService,
    network_name: String,
) -> Result<()> {
    let exists = docker
        .is_service_exists("traefik-service".to_string())
        .await;
    if !exists {
        let mut docker_params = ServiceParam::new(
            "traefik-service".to_string(),
            "traefik:v2.10".to_string(),
            network_name,
        );
        let args: Vec<String> = vec![
            "--providers.docker.swarmMode=true",
            "--api.insecure=true",
            "--metrics.prometheus.entrypoint=metrics",
            "--entrypoints.metrics.address=:8082",
            "--entrypoints.websecure.address=:443",
            "--entrypoints.web.address=:80",
            "--providers.docker.exposedbydefault=false",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        docker_params.add_args(args);

        docker_params.add_mount(ServiceMount::Bind(
            "/var/run/docker.sock".to_string(),
            "/var/run/docker.sock".to_string(),
        ));
        docker_params.add_port(80, 80);
        docker_params.add_port(443, 443);
        docker_params.add_port(8082, 8082);
        docker_params.add_port(8080, 8080);

        docker.create_service(docker_params).await?;
    }

    ok!(())
}
