use std::{env, fs::File, io::Write};

use anyhow::Result;
use shared::ok;

pub fn handle_new(name: Option<String>) -> Result<()> {
    let project_name = if name.is_some() {
        name.unwrap()
    } else {
        if let Ok(current_dir) = env::current_dir() {
            if let Some(folder_name) = current_dir.file_name() {
                let curdir = folder_name.to_string_lossy();
                curdir.to_string()
            } else {
                println!("No folder name found");
                "root".to_string()
            };
        }
        "root".to_string()
    };
    let config = get_initial_config(project_name);
    let mut file = File::create("deploy.yaml")?;
    file.write_all(config.as_bytes())?;
    ok!(())
}

fn get_initial_config(project_name: String) -> String {
    format!(
        r#"
project: {}

app:
  main:
    domain: example.com
    port: 3000
    "#,
        project_name
    )
}
