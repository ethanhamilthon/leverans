use anyhow::Result;
use chrono::{DateTime, Utc};
use shared::ok;
use sqlx::{prelude::FromRow, query, query_as, Executor, SqlitePool};
use uuid::Uuid;

pub const CONFIG_MIGRATION: &str = r#"
    create table if not exists configs (
        id text primary key,
        name text not null,
        config text not null,
        created_at text not null
    );
    "#;

#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow)]
pub struct ConfigData {
    pub id: String,
    pub name: String,
    pub config: String,
    pub created_at: String,
}

impl ConfigData {
    pub async fn migrate(conn: &SqlitePool) -> Result<()> {
        conn.execute(CONFIG_MIGRATION).await?;
        Ok(())
    }

    pub fn new(name: String, config: String) -> Result<Self> {
        ok!(Self {
            id: Uuid::new_v4().to_string(),
            name,
            config,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub async fn insert_db(self, conn: &SqlitePool) -> Result<Self> {
        query("insert into configs values (?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.name)
            .bind(&self.config)
            .bind(&self.created_at)
            .execute(conn)
            .await?;
        Ok(self)
    }

    pub async fn get_last_config(conn: &SqlitePool, name: &str) -> Result<Option<Self>> {
        let row = query_as::<_, ConfigData>(
            "select * from configs where name = ? order by created_at desc limit 1",
        )
        .bind(name)
        .fetch_optional(conn)
        .await?;

        Ok(row)
    }

    pub fn get_created_at(&self) -> Result<DateTime<Utc>> {
        let dt = DateTime::parse_from_rfc3339(&self.created_at)?;
        Ok(dt.with_timezone(&Utc))
    }
}
