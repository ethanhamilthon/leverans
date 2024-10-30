use std::time::Instant;

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use shared::ok;
use sqlx::{prelude::FromRow, query, query_as, Executor, SqlitePool};
use uuid::Uuid;

use crate::repo::Repo;

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub super_user: bool,
}

impl User {
    pub async fn migrate(conn: &SqlitePool) -> Result<()> {
        conn.execute(USER_MIGRATION).await?;
        Ok(())
    }
    pub fn new(username: String, password: String, super_user: bool) -> Result<Self> {
        ok!(Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash: hash(password, 4)?,
            super_user,
        })
    }

    pub async fn insert_db(self, conn: &SqlitePool) -> Result<Self> {
        query("insert into users values (?, ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.username)
            .bind(&self.password_hash)
            .bind(&self.super_user)
            .execute(conn)
            .await?;
        Ok(self)
    }

    pub async fn get_all(conn: &SqlitePool) -> Result<Vec<Self>> {
        let rows = query_as::<_, User>("select * from users")
            .fetch_all(conn)
            .await?;
        Ok(rows)
    }

    pub async fn get_by_id(id: &str, conn: &SqlitePool) -> Result<Self> {
        let row = query_as::<_, User>("select * from users where id = ?")
            .bind(id)
            .fetch_one(conn)
            .await?;
        Ok(row)
    }

    pub async fn get_by_username(username: &str, conn: &SqlitePool) -> Result<Self> {
        let row = query_as::<_, User>("select * from users where username = ?")
            .bind(username)
            .fetch_one(conn)
            .await?;
        Ok(row)
    }

    pub async fn super_user_exists(conn: &SqlitePool) -> Result<bool> {
        let rows = query_as::<_, User>("select * from users where super_user = 1")
            .fetch_all(conn)
            .await?;
        Ok(!rows.is_empty())
    }
}

pub const USER_MIGRATION: &str = r#"
    create table if not exists users (
        id text primary key,
        username text not null,
        password_hash text not null,
        super_user integer not null
    );
    "#;

#[tokio::test]
async fn user_repo() {
    let start = Instant::now();
    let pool = Repo::new(":memory:", true).await.unwrap();
    println!("Repo::new: {:?}", start.elapsed());

    let start = Instant::now();
    User::migrate(&pool.pool).await.unwrap();
    println!("User::migrate: {:?}", start.elapsed());

    let start = Instant::now();
    let user1 = User::new("test1".to_string(), "test1".to_string(), false)
        .unwrap()
        .insert_db(&pool.pool)
        .await
        .unwrap();
    println!("Insert user1: {:?}", start.elapsed());

    let start = Instant::now();
    let user2 = User::new("test2".to_string(), "test2".to_string(), true)
        .unwrap()
        .insert_db(&pool.pool)
        .await
        .unwrap();
    println!("Insert user2: {:?}", start.elapsed());

    let start = Instant::now();
    let user3 = User::new("test3".to_string(), "test3".to_string(), false)
        .unwrap()
        .insert_db(&pool.pool)
        .await
        .unwrap();
    println!("Insert user3: {:?}", start.elapsed());

    let start = Instant::now();
    let users = User::get_all(&pool.pool).await.unwrap();
    println!("User::get_all: {:?}", start.elapsed());

    let start = Instant::now();
    assert_eq!(users.len(), 3);
    println!("Assertions (users count): {:?}", start.elapsed());

    let start = Instant::now();
    assert!(users.iter().any(|u| u.id == user1.id));
    assert!(users.iter().any(|u| u.id == user2.id));
    assert!(users.iter().any(|u| u.id == user3.id));
    println!("Assertions (user IDs): {:?}", start.elapsed());
}
