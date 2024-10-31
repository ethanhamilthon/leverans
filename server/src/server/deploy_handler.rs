use std::{str::FromStr, sync::Arc};

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{
    config::MainConfig,
    ok,
    rollup::{rollupables::Rollupable, Rollup},
    SecretValue,
};

use crate::{
    repo::secret_repo::SecretData,
    server::auth_handler::{check_auth, must_auth},
};

use super::ServerData;

#[derive(Deserialize, Debug)]
pub struct ConfigBody {
    pub config: String,
}
pub async fn deploy_handle(
    sd: web::Data<Arc<ServerData>>,
    body: web::Json<ConfigBody>,
    req: HttpRequest,
) -> Result<impl Responder> {
    println!("deploying");
    println!("{:?}", body);
    must_auth(&req)?;
    let config = MainConfig::from_str(&body.config)
        .map_err(|e| InternalError::new(e, StatusCode::from_u16(400).unwrap()))?;

    let ras = Rollupable::new_from_config(config).unwrap();

    let rollup = Rollup::new(
        false,
        false,
        "aranea-network".to_string(),
        move |s| async move {
            println!("{}", s);
            Ok(())
        },
    )
    .map_err(|e| InternalError::new(e, StatusCode::from_u16(500).unwrap()))?;
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
    dbg!("secret_list: ", &secret_list);
    match rollup.rollup(ras, secret_list).await {
        Ok(_) => {
            println!("Rollup completed successfully");
            Ok(HttpResponse::Ok().body("Rollup completed successfully"))
        }
        Err(e) => {
            println!("Rollup error: {:?}", e);
            Ok(HttpResponse::InternalServerError().body(format!("Failed to rollup: {:?}", e)))
        }
    }
}
