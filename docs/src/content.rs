use anyhow::{anyhow, Result};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env, fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone)]
pub struct PageMetadata {
    pub title: String,
    pub path: String,
    pub folder: String,
    pub order: i32,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct PageElement {
    pub html: String,
    pub metadata: PageMetadata,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct MenuElement {
    pub title: String,
    pub path: String,
    pub order: i32,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct MenuFolder {
    pub folder_name: String,
    pub folder_title: String,
    pub elements: Vec<MenuElement>,
    pub order: i32,
}

pub fn collect_file_contents_in_dir<P: AsRef<Path>>(dir: P) -> Vec<String> {
    let mut file_contents = Vec::new();
    println!("opening files");
    for entry in WalkDir::new(dir) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    file_contents.push(content);
                }
            }
        }
    }

    file_contents
}

pub fn markdown_to_html(markdown: &str, default: bool) -> String {
    let parser = Parser::new(markdown);

    let mut html_output = String::new();
    if default {
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output
    } else {
        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Paragraph => html_output.push_str("<p class=\"mp\">"),
                    Tag::Heading { level, .. } => {
                        html_output.push_str(&format!("<{} class=\"m{} mhe\">", level, level))
                    }
                    Tag::Emphasis => html_output.push_str("<span class=\"mem\">"),
                    Tag::Strong => html_output.push_str("<strong class=\"ms\">"),
                    Tag::List(_) => html_output.push_str("<ul class=\"mul\">"), // Маркированный список
                    Tag::Item => html_output.push_str("<li class=\"mli\">"),
                    Tag::CodeBlock(info) => match info {
                        pulldown_cmark::CodeBlockKind::Indented => {
                            html_output.push_str(&format!("<span class=\"mcb mcbi\">"))
                        }
                        pulldown_cmark::CodeBlockKind::Fenced(cow_str) => html_output.push_str(
                            &format!("<span class=\"mcb mcbf\" data-lang=\"{}\">", cow_str),
                        ),
                    },
                    Tag::Link {
                        dest_url, title, ..
                    } => {
                        html_output.push_str(&format!(
                            "<a href=\"{}\" title=\"{}\" class=\"mlink\">",
                            dest_url, title
                        ));
                    }
                    _ => {}
                },

                Event::End(tag) => match tag {
                    TagEnd::Paragraph => html_output.push_str("</p>"),
                    TagEnd::Heading(level) => html_output.push_str(&format!("</{}>", level)),
                    TagEnd::Emphasis => html_output.push_str("</span>"),
                    TagEnd::Strong => html_output.push_str("</strong>"),
                    TagEnd::List(_) => html_output.push_str("</ul>"), // Закрытие маркированного списка
                    TagEnd::Item => html_output.push_str("</li>"),
                    TagEnd::CodeBlock => html_output.push_str("</span>"),
                    TagEnd::Link => html_output.push_str("</a>"),
                    _ => {}
                },
                Event::Text(text) => html_output.push_str(&text),
                Event::Code(text) => html_output
                    .push_str(format!("<span class=\"mcb mcbi\">{}</span>", text).as_str()),
                Event::Html(text) => html_output.push_str(&text),
                _ => {}
            }
        }

        html_output
    }
}
pub fn get_pages(root: &str) -> Result<Vec<PageElement>> {
    let mut pages = Vec::new();
    let file_contents = collect_file_contents_in_dir(root);

    for content in file_contents {
        let content_parts = content.split("---").collect::<Vec<&str>>();
        if content_parts.len() != 3 {
            continue;
        }

        let metadata_str = content_parts[1].trim();
        let metadata: PageMetadata = serde_yaml::from_str(metadata_str).unwrap();
        let html = markdown_to_html(content_parts[2].trim(), false);
        pages.push(PageElement { html, metadata });
    }

    Ok(pages)
}

pub fn get_menu(pages: Vec<PageElement>) -> Result<Vec<MenuFolder>> {
    let mut menu: Vec<MenuFolder> = Vec::new();
    menu.push(MenuFolder {
        folder_name: "start".to_string(),
        folder_title: "Getting started".to_string(),
        elements: vec![],
        order: 1,
    });
    menu.push(MenuFolder {
        folder_name: "config".to_string(),
        folder_title: "Configuration File".to_string(),
        elements: vec![],
        order: 2,
    });
    menu.push(MenuFolder {
        folder_name: "cli".to_string(),
        folder_title: "Command Line Interface".to_string(),
        elements: vec![],
        order: 3,
    });

    for page in pages {
        if let Some(folder) = menu
            .iter_mut()
            .find(|f| f.folder_name == page.metadata.folder)
        {
            folder.elements.push(MenuElement {
                title: page.metadata.title,
                path: format!("/{}/{}", folder.folder_name, page.metadata.path),
                order: page.metadata.order,
            });
        }
    }

    for folder in menu.iter_mut() {
        folder.elements.sort_by(|a, b| a.order.cmp(&b.order));
    }

    Ok(menu)
}

static GLOBAL_MAP: OnceLock<Arc<Mutex<HashMap<String, PageElement>>>> = OnceLock::new();
static GLOBAL_MENU: OnceLock<Arc<Mutex<Vec<MenuFolder>>>> = OnceLock::new();

pub fn get_root_path() -> Result<String> {
    let path = env::current_dir()?;
    let parsed_path = Path::new(&path);
    Ok(parsed_path
        .join("docs")
        .to_str()
        .ok_or(anyhow!("invalid path"))?
        .to_string())
}

pub fn get_global_map() -> Result<Arc<Mutex<HashMap<String, PageElement>>>> {
    let root = get_root_path()?;
    let map = GLOBAL_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        let pages = get_pages(&root).unwrap();
        for page in pages {
            map.insert(
                format!("/{}/{}", page.metadata.folder, page.metadata.path),
                page,
            );
        }
        Arc::new(Mutex::new(map))
    });
    Ok(map.clone())
}

pub fn get_global_menu() -> Result<Arc<Mutex<Vec<MenuFolder>>>> {
    let root = get_root_path()?;
    let menu = GLOBAL_MENU.get_or_init(|| {
        let pages = get_pages(&root).unwrap();
        let menu = get_menu(pages).unwrap();
        Arc::new(Mutex::new(menu))
    });
    Ok(menu.clone())
}

pub fn get_html_page(path: &str) -> Result<PageElement> {
    let map = get_global_map()?;
    let map = map.lock().map_err(|_| anyhow!("failed to lock map"))?;
    let page = map.get(path).ok_or(anyhow!("page not found"))?;
    Ok(page.clone())
}
