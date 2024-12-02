use actix_web::{web, HttpResponse, Responder, Result};
use regex::Regex;
use serde::Deserialize;

const INSTALL_CLI_SCRIPT: &str = include_str!("./scripts/unix.sh");
const INSTALL_WIN_CLI_SCRIPT: &str = include_str!("./scripts/windows.ps1");

#[derive(Deserialize)]
pub struct InstallCLIQuery {
    // version
    v: Option<String>,
    // email
    os: Option<String>,
}

pub async fn install_cli_script(
    _req: actix_web::HttpRequest,
    query: web::Query<InstallCLIQuery>,
) -> Result<impl Responder> {
    let version = if let Some(version) = query.v.clone() {
        if VERSIONS.contains(&version.as_str()) {
            version
        } else {
            VERSIONS[0].to_string()
        }
    } else {
        VERSIONS[0].to_string()
    };
    let script = if let Some(os) = query.os.clone() {
        if os == "windows" || os == "win" {
            INSTALL_WIN_CLI_SCRIPT
        } else {
            INSTALL_CLI_SCRIPT
        }
    } else {
        INSTALL_CLI_SCRIPT
    };

    let reg = Regex::new(r"acac").unwrap();
    let final_str = reg.replace_all(&script, &version).to_string();
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(final_str))
}

const INSTALL_MANAGER_SCRIPT: &str = include_str!("./scripts/manager.sh");

#[derive(Deserialize)]
pub struct InstallManagerQuery {
    // version
    v: Option<String>,
    // email
    e: Option<String>,
}

static VERSIONS: [&str; 2] = ["0.2.0", "0.1.9.1"];

pub async fn install_manager(
    _req: actix_web::HttpRequest,
    query: web::Query<InstallManagerQuery>,
) -> Result<impl Responder> {
    let version = if let Some(version) = query.v.clone() {
        if VERSIONS.contains(&version.as_str()) {
            version
        } else {
            VERSIONS[0].to_string()
        }
    } else {
        VERSIONS[0].to_string()
    };

    let email = query.e.clone().unwrap_or("admin@admin.com".to_string());
    let reg = Regex::new(r"acac").unwrap();
    let prefinal = reg
        .replace_all(&INSTALL_MANAGER_SCRIPT, &version)
        .to_string();
    let reg = Regex::new(r"abab").unwrap();
    let final_rep = reg.replace_all(&prefinal, &email).to_string();

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(final_rep))
}

const UNINSTALL_MANAGER_SCRIPT: &str = include_str!("./scripts/uninstall.sh");

pub async fn uninstall_manager(_req: actix_web::HttpRequest) -> Result<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(UNINSTALL_MANAGER_SCRIPT))
}
