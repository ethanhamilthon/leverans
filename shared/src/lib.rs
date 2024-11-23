use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub mod config;
pub mod console;
pub mod deployable;
pub mod docker;
pub mod docker_platform;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecretValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SmartString {
    This { service: String, method: String },
    Text(String),
    Secret(String),
}

impl SmartString {
    pub fn parse_env<'a>(value: &'a str) -> Result<Vec<SmartString>> {
        if value.chars().count() < 4 {
            ok!(vec![SmartString::Text(value.to_string())])
        }
        let value = value.trim();
        let first_2_char = &value[..2];
        let last_2_char = &value[value.len() - 2..value.len()];
        if first_2_char == "{{" && last_2_char == "}}" {
            let value = &value[2..value.len() - 2].trim();
            let text_parts: Vec<&str> = value.split('+').collect();
            let mut final_parts: Vec<SmartString> = Vec::new();
            for value in text_parts {
                let value = value.trim();
                if let Some(env_value) = value.strip_prefix(&"secret.") {
                    final_parts.push(SmartString::Secret(env_value.to_string()));
                    continue;
                }
                if let Some(env_value) = value.strip_prefix(&"this.") {
                    let parts: Vec<&str> = env_value.splitn(2, '.').collect();
                    if parts.len() != 2 {
                        err!(anyhow!(
                            "after \"this\" should be two arguments divided with \".\""
                        ))
                    }
                    final_parts.push(SmartString::This {
                        service: parts[0].to_string(),
                        method: parts[1].to_string(),
                    });
                    continue;
                }
                if value.starts_with("'") && value.ends_with("'") {
                    final_parts.push(SmartString::Text(value[1..value.len() - 1].to_string()));
                    continue;
                }
                dbg!(&value);
                err!(anyhow!("please use \"this\" or \"secret\""))
            }
            dbg!(&final_parts);
            ok!(final_parts)
        }
        ok!(vec![(SmartString::Text(value.to_string()))])
    }
}

pub fn get_unix_millis() -> u128 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

#[derive(Serialize, Deserialize)]
pub struct UserSafe {
    pub username: String,
    pub role: String,
}
