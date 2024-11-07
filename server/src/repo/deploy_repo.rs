use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use shared::ok;
use sqlx::{prelude::FromRow, query, query_as, Executor, SqlitePool};
use tokio::time::sleep;
use uuid::Uuid;

use crate::repo::Repo;

#[derive(Debug, serde::Serialize, serde::Deserialize, FromRow, PartialEq)]
pub struct DeployData {
    pub id: String,
    pub project_name: String,
    pub deploys: String,
    pub created_at: String,
}

const DEPLOY_MIGRATION: &str = r#"
    create table if not exists deploys (
        id text primary key,
        project_name text not null,
        deploys text not null,
        created_at text not null
    );
    "#;

impl DeployData {
    pub async fn migrate(conn: &SqlitePool) -> Result<()> {
        conn.execute(DEPLOY_MIGRATION).await?;
        Ok(())
    }

    pub fn new(project_name: String, deploys: String) -> Result<Self> {
        ok!(Self {
            id: Uuid::new_v4().to_string(),
            project_name,
            deploys,
            created_at: Utc::now().to_rfc3339(),
        })
    }

    pub async fn insert_db(&self, conn: &SqlitePool) -> Result<()> {
        query("insert into deploys values (?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.project_name)
            .bind(&self.deploys)
            .bind(&self.created_at)
            .execute(conn)
            .await?;
        Ok(())
    }
    pub async fn get_last_deploys(conn: &SqlitePool) -> Result<Vec<Self>> {
        let rows = query_as::<_, Self>(
            r#"SELECT id, project_name, deploys, created_at 
               FROM ( SELECT *, ROW_NUMBER() OVER (PARTITION BY project_name ORDER BY created_at DESC) 
               AS row_num FROM deploys ) AS subquery WHERE row_num = 1;"#,
        )
        .fetch_all(conn)
        .await?;
        Ok(rows)
    }
}

#[tokio::test]
async fn deploys_test() {
    let pool = Repo::new(":memory:", true).await.unwrap().pool;
    let deploys1 = DeployData::new("project1".to_string(), "deploys1".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let deploys1 = DeployData::new("project1".to_string(), "deploys2".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let deploys1 = DeployData::new("project1".to_string(), "deploys3".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let deploys1 = DeployData::new("project2".to_string(), "deploys4".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let deploys1 = DeployData::new("project2".to_string(), "deploys5".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let deploys1 = DeployData::new("project2".to_string(), "deploys6".to_string()).unwrap();
    let _ = deploys1.insert_db(&pool).await;
    sleep(Duration::from_millis(100)).await;
    let last_deploys = DeployData::get_last_deploys(&pool).await.unwrap();
    assert_eq!(last_deploys.len(), 2);
    assert!(last_deploys
        .iter()
        .any(|d| d.project_name == "project1" && d.deploys == "deploys3"));
    assert!(last_deploys
        .iter()
        .any(|d| d.project_name == "project2" && d.deploys == "deploys6"));
}
