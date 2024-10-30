pub mod user_repo;

use anyhow::Result;
use shared::{create_file_if_not_exist, ok};
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
        ok!(Self { pool })
    }
}
