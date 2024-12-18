use std::sync::Arc;

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{deployable::deploy::Deploy, err};

use crate::{
    repo::{deploy_repo::DeployData, user_repo::RoleType},
    server::auth_handler::must_auth,
};

use super::ServerData;

#[derive(Deserialize, Debug)]
pub struct ConfigBody {
    pub config: String,
    pub filter: Option<String>,
}

pub async fn handle_deploy(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<Vec<Deploy>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_auth(&req, vec![RoleType::FullAccess, RoleType::SuperUser])?;
    let mut project_name: String = String::new();
    let service_names: Vec<_> = sd
        .docker_service
        .list_services()
        .await
        .map_err(|_| {
            InternalError::new(
                "Failed to list services",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .filter_map(|s| {
            if let Some(spec) = s.spec {
                Some(spec.name.unwrap())
            } else {
                None
            }
        })
        .collect();
    for deploy in body.iter() {
        println!("deploying {}", deploy.deployable.short_name);
        if !project_name.is_empty() && project_name != deploy.deployable.project_name.clone() {
            err!(InternalError::new(
                "All deploys must be from the same project",
                StatusCode::from_u16(400).unwrap(),
            )
            .into());
        }
        project_name = deploy.deployable.project_name.clone();
        deploy
            .deploy(sd.docker_service.clone(), service_names.clone())
            .await
            .map_err(|e| {
                dbg!(&e);
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
