use actix_web::{HttpResponse, Responder, Result};

const INSTALL_CLI_SCRIPT: &str = r#"
#!/bin/bash
echo "Installing Lev CLI..."
"#;

pub async fn install_cli_script(req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(INSTALL_CLI_SCRIPT))
}
