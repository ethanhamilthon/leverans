use anyhow::Result;
use std::{fs, path::Path};

pub struct PageElement {
    pub html: String,
    pub title: String,
    pub path: String,
    pub number: i32,
    pub children: Vec<PageElement>,
    pub previous_name: Option<String>,
    pub previous_link: Option<String>,
    pub next_name: Option<String>,
    pub next_link: Option<String>,
}

pub fn get_pages(root_path: &Path) -> Result<Vec<PageElement>> {
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let typ = entry.file_type()?;
        if typ.is_dir() {
            let folder_name = entry.file_name().to_string_lossy().to_string();
            println!("Folder: {}", folder_name);
        } else {
            let file_name = entry.file_name().to_string_lossy().to_string();
            println!("File: {}", file_name);
        }
    }
    Ok(vec![])
}

#[test]
fn test_get_pages() {
    let root_path = Path::new("./");
    let pages = get_pages(root_path).unwrap();
    assert!(pages.is_empty());
}
