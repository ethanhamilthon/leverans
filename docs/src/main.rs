use actix_web::{get, web::Html, App, HttpServer, Responder};
use askama::Template;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[get("/")]
async fn index() -> impl Responder {
    // let markdown_input = "# Пример заголовка\n\nТекст **с жирным выделением** и *курсивом*.";
    // let html_output = parse_mark(markdown_input);
    let layout = HelloTemplate {
        title: "<strong>Test</strong>".to_string(),
    };

    Html::new(layout.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind(("127.0.0.1", 8082))?
        .run()
        .await
}

#[derive(Template)] // this will generate the code...
#[template(path = "layout.html")] // using the template in this path, relative
struct HelloTemplate {
    pub title: String,
}

fn parse_mark(input: &str) -> String {
    let parser = Parser::new(input);

    let mut html_output = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => html_output.push_str("<p>"),
                Tag::Heading { level, .. } => html_output.push_str(&format!("<{}>", level)),
                Tag::Emphasis => html_output.push_str("<em>"),
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
