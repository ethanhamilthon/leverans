use anyhow::{anyhow, Result};
use shared::{create_file_with_dirs, err, get_home_path, ok};
use sqlx::{query, query_as, sqlite::SqlitePool, Executor};

const DATABASE_URI_FOR_FILE: &str = ".config/leverans/leverans.db";

pub struct UserData {
    pub pool: SqlitePool,
}

impl UserData {
    pub async fn load_current_user(&self) -> Result<RemoteAuth> {
        let remote_auths =
            query_as::<_, RemoteAuth>("select id, remote_url, remote_token from remote_auth")
                .fetch_all(&self.pool)
                .await?;
        match remote_auths.len() {
            1 => ok!(remote_auths[0].clone()),
            0 => err!(anyhow!("There is no remote auth set in the database")),
            _ => err!(anyhow!("There are more that 1 row, this should not happen")),
        }
    }

    pub async fn save_user(&self, token: String, url: String) -> Result<()> {
        query("insert into remote_auth (remote_url, remote_token) values ( ?, ?)")
            .bind(url)
            .bind(token)
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
        let url = get_home_path(DATABASE_URI_FOR_FILE)?
            .to_str()
            .ok_or(anyhow!("Failed to convert path to string"))?
            .to_owned();
        if !is_memory {
            create_file_with_dirs(&url)?;
        }
        let pool = SqlitePool::connect(if is_memory { ":memory:" } else { &url }).await?;
        pool.execute(MIGRATION).await?;
        ok!(Self { pool })
    }
}

const MIGRATION: &str = r#"
    create table if not exists remote_auth (
        id integer primary key,
        remote_url text not null,
        remote_token text not null
    );
    "#;

#[derive(sqlx::FromRow, Clone)]
pub struct RemoteAuth {
    pub id: i64,
    pub remote_url: String,
    pub remote_token: String,
}

#[tokio::test]
async fn test_load_db() {
    let ud = UserData::load_db(true).await.expect("Failed to load db");
    ud.save_user("sometoken".to_string(), "someurl".to_string())
        .await
        .expect("Failed to save user");

    let current_user = ud.load_current_user().await.expect("Failed to load user");
    assert_eq!(current_user.remote_url, "someurl");
    assert_eq!(current_user.remote_token, "sometoken");
}
