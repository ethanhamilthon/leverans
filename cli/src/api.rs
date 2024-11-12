use reqwest::{multipart::Form, StatusCode};
use serde_json::{self, json};

use anyhow::{anyhow, Result};
use shared::{deployable::deploy::Deploy, err, ok, Secret, UserAuthBody};
use url::Url;

pub struct API {
    pub main_url: Url,
    pub req_client: reqwest::Client,
}

impl API {
    pub fn new(url: &str) -> Result<Self> {
        //println!("Connected to {}", url);
        Ok(API {
            main_url: Url::parse(url)?,
            req_client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // Отключаем проверку сертификатов
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
        })
    }

    pub async fn upload_config(
        &self,
        config: String,
        token: String,
        filter: Option<String>,
    ) -> Result<()> {
        let mut upload_url = self.main_url.clone();
        upload_url.set_path("/deploy");
        let body = json!({
            "config": config,
            "filter": filter
        })
        .to_string();
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

    pub async fn deploy_plan(&self, deploys: Vec<Deploy>, token: String) -> Result<()> {
        let mut upload_url = self.main_url.clone();
        upload_url.set_path("/new-deploy");

        let body = serde_json::to_string(&deploys)?;
        //println!("{}", body);
        let res = self
            .req_client
            .post(upload_url)
            .body(body)
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to deploy plan: {}", error_text))
        }
    }

    pub async fn get_plans(
        &self,
        config: String,
        token: String,
        to_build: Option<Vec<String>>,
        filter: Vec<String>,
    ) -> Result<Vec<Deploy>> {
        let mut upload_url = self.main_url.clone();
        upload_url.set_path("/plan");
        let body = json!({
            "config": config,
            "filter": filter,
            "to_build": to_build
        })
        .to_string();
        let res = self
            .req_client
            .get(upload_url)
            .body(body)
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .send()
            .await?;

        if res.status().is_success() {
            let plans = serde_json::from_str::<Vec<Deploy>>(&res.text().await?)?;
            Ok(plans)
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to get plans: {}", error_text))
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

    pub async fn add_secret(&self, key: &str, value: &str, token: &str) -> Result<()> {
        let mut super_user_url = self.main_url.clone();
        super_user_url.set_path("/secret");
        let res = self
            .req_client
            .post(super_user_url)
            .body(
                json!({
                    "key": key,
                    "value": value
                })
                .to_string(),
            )
            .header("Content-Type", "application/json")
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(())
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to add secret: {}", error_text))
        }
    }

    pub async fn list_secret(&self, token: &str) -> Result<Vec<Secret>> {
        let mut super_user_url = self.main_url.clone();
        super_user_url.set_path("/secret");
        let res = self
            .req_client
            .get(super_user_url)
            .header("X-LEVERANS-PASS", "true")
            .header("Authorization", token)
            .send()
            .await?;
        if res.status().is_success() {
            let text = res.text().await?;
            let secrets: Vec<Secret> = serde_json::from_str(&text)?;
            Ok(secrets)
        } else {
            let error_text = res.text().await?;
            Err(anyhow!("Failed to list secret: {}", error_text))
        }
    }
}

#[tokio::test]
async fn health_test() {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // отключаем проверку сертификатов
        .build()
        .unwrap();
    let res = client
        .get("https://localhost/healthz")
        .header("X-LEVERANS-PASS", "true")
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());
}
