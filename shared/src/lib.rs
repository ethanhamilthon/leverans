use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod config;
pub mod config_process;
pub mod console;
pub mod docker;
pub mod docker_platform;
pub mod rollup;

#[macro_export]
macro_rules! err {
    ($e:expr) => {
        return Err($e)
    };
}

#[macro_export]
macro_rules! ok {
    ($e:expr) => {
        return Ok($e)
    };
}

#[macro_export]
macro_rules! strf {
    ($s:expr) => {{
        // Проверяем, что $s имеет тип &str
        let temp: &str = $s;
        String::from(temp)
    }};
}

pub fn can_be(strg: Option<String>, vars: Vec<String>) -> bool {
    if strg.is_none() {
        return false;
    }
    let str_to_vl = strg.unwrap();
    if vars.is_empty() || !vars.contains(&str_to_vl) {
        return false;
    }
    true
}

pub trait GlobalError {
    fn read_cause(&self) -> String;
}

pub static NETWORK_NAME: &str = "aranea-network";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserAuthBody {
    pub username: String,
    pub password: String,
}

impl UserAuthBody {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

pub fn get_home_path(file_path: &str) -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or(anyhow!("Failed to get home dir"))?;
    path.push(file_path);
    ok!(path)
}

pub fn create_file_with_dirs(path: &str) -> Result<()> {
    let path = Path::new(path);
    let dir_path = path.parent().ok_or(anyhow!("Failed to get parent dir"))?;

    fs::create_dir_all(dir_path)?;

    if !path.exists() {
        create_file_if_not_exist(
            path.to_str()
                .ok_or(anyhow!("Failed to convert path to string"))?,
        )?;
    }
    ok!(())
}

pub fn create_file_if_not_exist(path: &str) -> Result<()> {
    if !std::path::Path::new(path).exists() {
        std::fs::File::create(path)?;
    }
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Secret {
    pub key: String,
    pub created_at: String,
}
