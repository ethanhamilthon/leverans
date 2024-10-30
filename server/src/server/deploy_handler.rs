use std::str::FromStr;

use actix_web::{
    error::InternalError, http::StatusCode, web, HttpRequest, HttpResponse, Responder, Result,
};
use serde::Deserialize;
use shared::{
    config::MainConfig,
    ok,
    rollup::{rollupables::Rollupable, Rollup},
};

use crate::server::auth_handler::{check_auth, must_auth};

#[derive(Deserialize, Debug)]
pub struct ConfigBody {
    pub config: String,
}
pub async fn deploy_handle(
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
    match rollup.rollup(ras).await {
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
