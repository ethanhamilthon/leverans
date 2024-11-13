use anyhow::Result;
use std::collections::HashMap;

trait PushStr {
    fn push_str(&mut self, k: &str, v: &str);
}

impl PushStr for HashMap<String, String> {
    fn push_str(&mut self, k: &str, v: &str) {
        self.insert(k.to_string(), v.to_string());
    }
}

fn open_file_str(folder_path: &str, file_name: &str) -> Result<String> {
    let file_path = format!("{}/{}", folder_path, file_name);
    std::fs::read_to_string(file_path).map_err(|e| e.into())
}

pub fn get_content(folder_path: &str) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    map.push_str("overview", &open_file_str(folder_path, "overview.md")?);
    map.push_str("install", &open_file_str(folder_path, "install.md")?);
    Ok(map)
}
