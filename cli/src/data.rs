use std::fs;

use anyhow::{anyhow, Result};
use shared::{create_file_with_dirs, err, get_home_path, ok};
use sqlx::{query, query_as, sqlite::SqlitePool, Executor};

const DATABASE_URI_FOR_FILE: &str = ".config/leverans/leverans.db";

pub struct UserData {
    pub pool: SqlitePool,
}

impl UserData {
    pub async fn load_current_user(&self) -> Result<RemoteAuth> {
        let remote_auths = query_as::<_, RemoteAuth>(
            "select id, remote_url, remote_token, username from remote_auth",
        )
        .fetch_all(&self.pool)
        .await?;
        match remote_auths.len() {
            1 => ok!(remote_auths[0].clone()),
            0 => err!(anyhow!("There is no remote auth set in the database")),
            _ => err!(anyhow!("There are more that 1 row, this should not happen")),
        }
    }

    pub async fn save_user(&self, token: String, url: String, username: String) -> Result<()> {
        query("insert into remote_auth (remote_url, remote_token, username) values ( ?, ?, ? )")
            .bind(url)
            .bind(token)
            .bind(username)
            .execute(&self.pool)
            .await?;
        ok!(())
    }

    pub async fn delete_user(&self, id: i64) -> Result<()> {
        query("delete from remote_auth where id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        ok!(())
    }

    pub async fn load_db(is_memory: bool) -> Result<Self> {
        let url = if is_memory {
            ":memory:".to_string()
        } else {
            let home_dir = dirs::home_dir().expect("Не удалось получить домашнюю директорию");

            // Создать путь к папке "lev"
            let lev_path = home_dir.join("lev");

            // Создать папку, если она не существует
            if !lev_path.exists() {
                fs::create_dir(&lev_path)?;
            }

            // Создать файл "main.db" внутри папки "lev"
            let db_path = lev_path.join("main.db");
            if !db_path.exists() {
                fs::File::create(db_path.clone())?;
            }

            db_path.to_str().unwrap().to_string()
        };
        let pool = SqlitePool::connect(&url).await?;
        pool.execute(MIGRATION).await?;
        ok!(Self { pool })
    }
}

const MIGRATION: &str = r#"
    create table if not exists remote_auth (
        id integer primary key,
        remote_url text not null,
        remote_token text not null,
        username text not null
    );
    "#;

#[derive(sqlx::FromRow, Clone)]
pub struct RemoteAuth {
    pub id: i64,
    pub remote_url: String,
    pub remote_token: String,
    pub username: String,
}

#[tokio::test]
async fn test_load_db() {
    let ud = UserData::load_db(true).await.expect("Failed to load db");
    ud.save_user(
        "sometoken".to_string(),
        "someurl".to_string(),
        "username".to_string(),
    )
    .await
    .expect("Failed to save user");

    let current_user = ud.load_current_user().await.expect("Failed to load user");
    assert_eq!(current_user.remote_url, "someurl");
    assert_eq!(current_user.remote_token, "sometoken");
}
