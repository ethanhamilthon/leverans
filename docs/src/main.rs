use actix_files as fs;
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder, Result,
};
use askama::Template;
use content::{get_global_menu, get_html_page, MenuFolder};
use install::{install_cli_script, install_manager, uninstall_manager};

pub mod content;
pub mod install;

async fn index(req: actix_web::HttpRequest) -> Result<impl Responder> {
    let now = std::time::Instant::now();
    let path = req.path().to_string();
    let page = match get_html_page(&path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(HttpResponse::Found()
                .append_header(("LOCATION", "/start/get-started"))
                .finish())
        }
    };
    let folders = get_global_menu().unwrap().lock().unwrap().to_vec();
    let index = IndexTemplate {
        title: format!("{} | Lev Docs", page.metadata.title.clone()),
        content: page.html.clone(),
        folders,
    };
    println!("rendering index in {}ms", now.elapsed().as_millis());
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render().unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting in http://localhost:8000/");
    HttpServer::new(move || {
        App::new()
            .route("/install", web::get().to(install_cli_script))
            .route("/manager.sh", web::get().to(install_manager))
            .route("/uninstall.sh", web::get().to(uninstall_manager))
            .default_service(web::route().to(index))
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub content: String,
    pub folders: Vec<MenuFolder>,
}

#[derive(Template)]
#[template(path = "header.html")]
pub struct HeaderTemplate {
    pub title: String,
}
