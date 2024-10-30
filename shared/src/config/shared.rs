use crate::{err, ok, strf};

use super::MainConfig;

#[derive(Debug, PartialEq)]
pub enum EnvValueType<'a> {
    This { service: &'a str, method: &'a str },
    Text(&'a str),
    Secret(&'a str),
}

#[derive(Debug)]
pub enum ConfigError {
    EnvParse(&'static str),
}

impl MainConfig {
    pub fn exists_in_project(&self, type_of: &str, value: &str) -> bool {
        let exists = match type_of {
            "app" => {
                let apps = self.app.as_ref();
                return match apps {
                    Some(app) => return app.contains_key(value),
                    None => false,
                };
            }
            "db" => {
                let dbs = self.db.as_ref();
                return match dbs {
                    Some(db) => return db.contains_key(value),
                    None => false,
                };
            }
            "service" => true,
            "stc" => true,
            _ => false,
        };
        exists
    }

    pub fn parse_env<'a>(value: &'a str) -> Result<EnvValueType, ConfigError> {
        if value.chars().count() < 4 {
            ok!(EnvValueType::Text(value))
        }
        let value = value.trim();
        let first_2_char = &value[..2];
        let last_2_char = &value[value.len() - 2..value.len()];
        if first_2_char == "{{" && last_2_char == "}}" {
            let value = &value[2..value.len() - 2].trim();
            if let Some(env_value) = value.strip_prefix("secret.") {
                ok!(EnvValueType::Secret(env_value))
            }
            if let Some(env_value) = value.strip_prefix("this.") {
                let parts: Vec<&str> = env_value.splitn(2, '.').collect();
                if parts.len() != 2 {
                    err!(ConfigError::EnvParse(
                        "after \"this\" should be two arguments divided with \".\""
                    ))
                }
                ok!(EnvValueType::This {
                    service: parts[0],
                    method: parts[1]
                })
            }
            err!(ConfigError::EnvParse("please use \"this\" or \"secret\""))
        }
        ok!(EnvValueType::Text(value))
    }
}

#[test]
fn env_parse_test() {
    let result = MainConfig::parse_env(" {{ secret.mysec  }}  ").unwrap();
    assert_eq!(result, EnvValueType::Secret("mysec"));

    let result = MainConfig::parse_env("  some value   ").unwrap();
    assert_eq!(result, EnvValueType::Text("some value"));

    let result = MainConfig::parse_env("{{   this.myapp.app    }}").unwrap();
    assert_eq!(
        result,
        EnvValueType::This {
            service: "myapp",
            method: "app"
        }
    )
}
