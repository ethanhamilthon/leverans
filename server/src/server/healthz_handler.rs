use actix_web::{HttpRequest, HttpResponse, Responder, Result};
use shared::ok;

use crate::server::auth_handler::must_have_levpass;

pub async fn handle_healthz(req: HttpRequest) -> Result<impl Responder> {
    must_have_levpass(&req)?;
    ok!(HttpResponse::Ok().body("OK"))
}
