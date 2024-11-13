use actix_files as fs;
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder, Result,
};
use askama::Template;
use content::get_html_page;

pub mod content;
pub mod render;

async fn index(req: actix_web::HttpRequest) -> Result<impl Responder> {
    let path = req.path().to_string();
    let files = match get_html_page(&path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(HttpResponse::Found()
                .append_header(("LOCATION", "/start/overview"))
                .finish())
        }
    };
    let index = IndexTemplate {
        title: "Home".to_string(),
        content: files,
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
}

#[derive(Template)]
#[template(path = "header.html")]
pub struct HeaderTemplate {
    pub title: String,
}
