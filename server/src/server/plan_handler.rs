use std::sync::Arc;

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{
    deployable::{
        deploy::{plan, PlanParamaters},
        rollback::{rollback, RollBackParams},
    },
    ok, SecretValue,
};

use crate::repo::{deploy_repo::DeployData, secret_repo::SecretData, user_repo::RoleType};

use super::{auth_handler::must_auth, ServerData};

#[derive(Deserialize, Debug)]
pub struct PlanBody {
    pub config: String,
    pub filter: Option<Vec<String>>,
    pub to_build: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct RollBackBody {
    pub config: String,
}

pub async fn handle_rollback(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<RollBackBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_auth(
        &req,
        vec![
            RoleType::FullAccess,
            RoleType::SuperUser,
            RoleType::UpdateOnly,
            RoleType::ReadOnly,
        ],
    )?;
    let last_deploys: Vec<_> = DeployData::get_last_deploys(&sd.repo.pool, 1)
        .await
        .map_err(|e| {
            dbg!(e);
            InternalError::new(
                "Failed to get last deploys",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .map(|d| (d.project_name, d.deploys))
        .collect();
    let prelast_deploys: Vec<_> = DeployData::get_last_deploys(&sd.repo.pool, 2)
        .await
        .map_err(|e| {
            dbg!(e);
            InternalError::new(
                "Failed to get last deploys",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .map(|d| (d.project_name, d.deploys))
        .collect();

    let params = RollBackParams {
        main_config: body.config.clone(),
        last_deploys,
        prelast_deploys,
    };
    let final_deploys = rollback(params).map_err(|e| {
        dbg!(e);
        InternalError::new(
            "Failed to get last deploys",
            StatusCode::from_u16(500).unwrap(),
        )
    })?;
    Ok(HttpResponse::Ok().json(final_deploys))
}

pub async fn handle_plan(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<PlanBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    must_auth(
        &req,
        vec![
            RoleType::FullAccess,
            RoleType::SuperUser,
            RoleType::UpdateOnly,
            RoleType::ReadOnly,
        ],
    )?;
    dbg!(&body);
    let secrets: Vec<_> = SecretData::list_db(&sd.repo.pool)
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
    let deploys: Vec<_> = DeployData::get_last_deploys(&sd.repo.pool, 1)
        .await
        .map_err(|e| {
            dbg!(e);
            InternalError::new(
                "Failed to get last deploys",
                StatusCode::from_u16(500).unwrap(),
            )
        })?
        .into_iter()
        .map(|d| (d.project_name, d.deploys))
        .collect();
    let images: Vec<_> = sd
        .docker_service
        .list_images()
        .await
        .map_err(|_| {
            InternalError::new("Failed to get images", StatusCode::from_u16(500).unwrap())
        })?
        .into_iter()
        .map(|i| i.tag)
        .collect();
    let params = PlanParamaters {
        main_config: body.config.clone(),
        last_deploys: deploys,
        secrets,
        network_name: "lev".to_string(),
        filter: body.filter.clone(),
        to_build: body.to_build.clone().unwrap_or(vec![]),
        images,
    };
    let this_deploys = plan(params)
        .map_err(|e| InternalError::new(format!("{}", e), StatusCode::from_u16(400).unwrap()))?;
    ok!(HttpResponse::Ok().json(this_deploys))
}
