use std::{str::FromStr, sync::Arc};

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{
    config::MainConfig,
    deployable::deploy::{Deploy, DeployParameters},
    err, ok, SecretValue,
};

use crate::{
    repo::{config_repo::ConfigData, deploy_repo::DeployData, secret_repo::SecretData},
    server::auth_handler::{check_auth, must_auth},
};

use super::ServerData;

#[derive(Deserialize, Debug)]
pub struct ConfigBody {
    pub config: String,
    pub filter: Option<String>,
}

pub async fn new_deploy_handle(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<ConfigBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    println!("deploying new handler");
    println!("{:?}", body);
    println!("with filter : {}", body.filter.is_some());
    must_auth(&req)?;
    let project_name = MainConfig::from_str(&body.config)
        .map_err(|e| InternalError::new(e, StatusCode::from_u16(400).unwrap()))?
        .project;
    let last_config = ConfigData::get_last_config(&sd.repo.pool, project_name.as_str())
        .await
        .map_err(|_| {
            InternalError::new(
                "Failed to get last config",
                StatusCode::from_u16(500).unwrap(),
            )
        })?;
    let secret_list: Vec<SecretValue> = SecretData::list_db(&sd.repo.pool)
        .await
        .map_err(|_| {
            InternalError::new(
                "Failed to get secret list",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .map(|s| SecretValue {
            key: s.key,
            value: s.value,
        })
        .collect();
    let deploy_res = DeployParameters {
        main_config: body.config.clone(),
        last_config: last_config.map(|c| c.config),
        secrets: secret_list,
        docker: sd.docker_service.clone(),
        is_local: true,
        network_name: "lev".to_string(),
        filter: body.filter.clone(),
    }
    .deploy()
    .await;
    match deploy_res {
        Ok(deps) => {
            println!("Deployed successfully");
            ConfigData::new(project_name, deps)
                .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?
                .insert_db(&sd.repo.pool)
                .await
                .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?;
            Ok(HttpResponse::Ok().body("Deployed successfully"))
        }
        Err(e) => {
            println!("Deploy error: {:?}", e);
            Ok(HttpResponse::InternalServerError().body(format!("Failed to deploy: {:?}", e)))
        }
    }
}

pub async fn handle_deploy(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<Vec<Deploy>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_auth(&req)?;
    let mut project_name: String = String::new();
    for deploy in body.iter() {
        if !project_name.is_empty() && project_name != deploy.deployable.project_name.clone() {
            err!(InternalError::new(
                "All deploys must be from the same project",
                StatusCode::from_u16(400).unwrap(),
            )
            .into());
        }
        project_name = deploy.deployable.project_name.clone();
        deploy
            .deploy(sd.docker_service.clone())
            .await
            .map_err(|e| {
                InternalError::new(
                    format!("Failed to deploy: {:?}", e),
                    StatusCode::from_u16(500).unwrap(),
                )
            })?;
    }
    DeployData::new(
        project_name,
        serde_json::to_string(&body)
            .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?,
    )
    .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?
    .insert_db(&sd.repo.pool)
    .await
    .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?;
    println!("Deployed successfully");
    Ok(HttpResponse::Ok().body("Deployed successfully"))
}
