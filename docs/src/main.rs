use std::{collections::HashMap, sync::Arc};

use actix_files as fs;
use actix_web::{
    error::InternalError,
    get,
    http::StatusCode,
    web::{self, Html},
    App, HttpServer, Responder, Result,
};
use askama::Template;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use render::get_content;

pub mod content;
pub mod render;

#[get("/c/{name}")]
async fn hello(
    name: web::Path<String>,
    files: web::Data<Arc<HashMap<String, String>>>,
) -> Result<impl Responder> {
    let file = files
        .get(name.as_str())
        .ok_or(InternalError::new("file not found", StatusCode::NOT_FOUND))?;
    let html_output = parse_mark(file);
    let layout = IndexTemplate {
        title: format!("Docs - {}", capitalize_first_letter(name.as_str())),
        content: html_output,
    };

    Ok(Html::new(layout.render().unwrap()))
}

#[get("/")]
async fn index() -> impl Responder {
    let markdown_input = "# Пример заголовка\n\nТекст **с жирным выделением** и *курсивом*.";
    let html_output = parse_mark(markdown_input);
    let layout = IndexTemplate {
        title: "Docs".to_string(),
        content: html_output,
    };

    Html::new(layout.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting in http://localhost:8082/");
    let files = Arc::new(get_content("./content").expect("Failed to get files"));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(files.clone()))
            .service(index)
            .service(hello)
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

pub fn parse_mark(input: &str) -> String {
    let parser = Parser::new(input);

    let mut html_output = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => html_output.push_str("<p class=\"mp\">"),
                Tag::Heading { level, .. } => {
                    html_output.push_str(&format!("<{} class=\"m{} mhe\">", level, level))
                }
                Tag::Emphasis => html_output.push_str("<em class=\"mem\">"),
                Tag::Strong => html_output.push_str("<strong>"),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => html_output.push_str("</p>"),
                TagEnd::Heading(level) => html_output.push_str(&format!("</{}>", level)),
                TagEnd::Emphasis => html_output.push_str("</em>"),
                TagEnd::Strong => html_output.push_str("</strong>"),
                _ => {}
            },
            Event::Text(text) => html_output.push_str(&text),
            _ => {}
        }
    }

    html_output
}

fn capitalize_first_letter(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
