use std::{str::FromStr, sync::Arc};

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{config::MainConfig, deployable::deploy::DeployParameters, ok, SecretValue};

use crate::{
    repo::{config_repo::ConfigData, secret_repo::SecretData},
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
