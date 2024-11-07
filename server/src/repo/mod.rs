pub mod config_repo;
pub mod deploy_repo;
pub mod secret_repo;
pub mod user_repo;

use anyhow::Result;
use config_repo::ConfigData;
use deploy_repo::DeployData;
use secret_repo::SecretData;
use shared::{create_file_if_not_exist, ok, Secret};
use sqlx::{query, sqlite::SqlitePool, Executor};
use user_repo::User;

#[derive(Clone, Debug)]
pub struct Repo {
    pub pool: SqlitePool,
}

impl Repo {
    pub async fn new(url: &str, is_memory: bool) -> Result<Self> {
        if !is_memory {
            create_file_if_not_exist(url)?;
        }
        let pool = SqlitePool::connect(if is_memory { ":memory:" } else { url }).await?;
        User::migrate(&pool).await?;
        SecretData::migrate(&pool).await?;
        ConfigData::migrate(&pool).await?;
        DeployData::migrate(&pool).await?;
        ok!(Self { pool })
    }
}
