use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{prelude::FromRow, query, query_as, Executor, SqlitePool};
use uuid::Uuid;

use crate::repo::Repo;

#[derive(Clone, Debug, FromRow)]
pub struct SecretData {
    pub id: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
}

impl SecretData {
    pub fn new(key: String, value: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            key,
            value,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub async fn migrate(conn: &SqlitePool) -> Result<()> {
        conn.execute(SECRET_MIGRATION).await?;
        Ok(())
    }

    pub async fn insert_db(self, conn: &SqlitePool) -> Result<()> {
        query("insert into secrets values (?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.key)
            .bind(&self.value)
            .bind(&self.created_at)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn list_db(conn: &SqlitePool) -> Result<Vec<Self>> {
        let rows = query_as::<_, SecretData>("select * from secrets")
            .fetch_all(conn)
            .await?;
        Ok(rows)
    }

    pub fn get_created_at(&self) -> Result<DateTime<Utc>> {
        let dt = DateTime::parse_from_rfc3339(&self.created_at)?;
        Ok(dt.with_timezone(&Utc))
    }

    pub async fn delete_db(key: String, conn: &SqlitePool) -> Result<()> {
        query("delete from secrets where key = ?")
            .bind(key)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn update_db(key: String, value: String, conn: &SqlitePool) -> Result<()> {
        query("update secrets set value = ? where key = ?")
            .bind(value)
            .bind(key)
            .execute(conn)
            .await?;
        Ok(())
    }
}

pub const SECRET_MIGRATION: &str = r#"
    create table if not exists secrets (
        id text primary key,
        key text not null,
        value text not null,
        created_at text not null
    )
    "#;
#[tokio::test]
async fn test_secret_repo() {
    let secret = SecretData::new("key".to_string(), "value".to_string());
    let pool = Repo::new("", true).await.unwrap();
    secret.insert_db(&pool.pool).await.unwrap();
    let rows = SecretData::list_db(&pool.pool).await.unwrap();
    assert_eq!(rows.len(), 1);
    let sec = rows[0].clone();
    sec.get_created_at().unwrap();
}
