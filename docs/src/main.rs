use actix_files as fs;
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder, Result,
};
use askama::Template;
use content::{get_global_menu, get_html_page, MenuFolder};

pub mod content;
pub mod install;

async fn install_script() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().content_type("text/plain").body(
        r#"
    #!/bin/bash
    echo "Installing Lev Docs..."
    "#,
    ))
}

async fn index(req: actix_web::HttpRequest) -> Result<impl Responder> {
    let path = req.path().to_string();
    let page = match get_html_page(&path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(HttpResponse::Found()
                .append_header(("LOCATION", "/start/overview"))
                .finish())
        }
    };
    let folders = get_global_menu().unwrap().lock().unwrap().to_vec();
    let index = IndexTemplate {
        title: format!("{} | Lev Docs", page.metadata.title.clone()),
        content: page.html.clone(),
        folders,
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render().unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting in http://localhost:8082/");
    HttpServer::new(move || {
        App::new()
            .route("/install.sh", web::get().to(install_script))
            .default_service(web::route().to(index))
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8082))?
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
