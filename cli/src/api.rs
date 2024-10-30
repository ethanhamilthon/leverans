use reqwest::{multipart::Form, StatusCode};
use serde_json;
use std::{borrow::Borrow, collections::HashMap};

use anyhow::{anyhow, Result};
use shared::{err, ok, UserAuthBody};
use url::Url;

pub struct API {
    pub main_url: Url,
    pub req_client: reqwest::Client,
}

impl API {
    pub fn new(url: &str) -> Result<Self> {
        Ok(API {
            main_url: Url::parse(url)?,
            req_client: reqwest::Client::new(),
        })
    }

    pub async fn upload_config(&self, config: String, token: String) -> Result<()> {
        let mut upload_url = self.main_url.clone();
        upload_url.set_path("/deploy");
        let body = serde_json::to_string(&HashMap::from([("config", config)])).unwrap();
        let res = self
            .req_client
            .post(upload_url)
            .body(body)
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to upload config: {}", error_text))
        }
    }

    pub async fn upload_image(&self, form: Form, token: String) -> Result<()> {
        let mut upload_url = self.main_url.clone();
        upload_url.set_path("/upload_image");

        let res = self
            .req_client
            .post(upload_url)
            .multipart(form)
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to upload image: {}", error_text))
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let mut health_check_url = self.main_url.clone();
        health_check_url.set_path("/healthz");
        let res = self
            .req_client
            .get(health_check_url)
            .header("X-LEVERANS-PASS", "true")
            .send()
            .await?;
        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to check health: {}", error_text))
        }
    }

    pub async fn is_super_user_exists(&self) -> Result<bool> {
        let mut super_user_url = self.main_url.clone();
        super_user_url.set_path("/auth/super");
        let res = self
            .req_client
            .get(super_user_url)
            .header("X-LEVERANS-PASS", "true")
            .send()
            .await?;
        match res.status() {
            StatusCode::OK => ok!(true),
            StatusCode::NOT_FOUND => ok!(false),
            _ => {
                let error_text = res.text().await?;
                err!(anyhow!("Failed to get super user status: {}", error_text))
            }
        }
    }

    pub async fn register_super_user(&self, username: &str, password: &str) -> Result<String> {
        let mut super_user_url = self.main_url.clone();
        super_user_url.set_path("/register/super");
        let res = self
            .req_client
            .post(super_user_url)
            .body(
                UserAuthBody {
                    username: username.to_string(),
                    password: password.to_string(),
                }
                .to_json()?,
            )
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.text().await?)
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to register super user: {}", error_text))
        }
    }

    pub async fn login_user(&self, username: &str, password: &str) -> Result<String> {
        let mut super_user_url = self.main_url.clone();
        super_user_url.set_path("/login/super");
        let res = self
            .req_client
            .post(super_user_url)
            .body(
                UserAuthBody {
                    username: username.to_string(),
                    password: password.to_string(),
                }
                .to_json()?,
            )
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.text().await?)
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to login super user: {}", error_text))
        }
    }
}
