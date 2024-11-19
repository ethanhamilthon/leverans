use std::time::Instant;

use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use shared::ok;
use sqlx::{prelude::FromRow, query, query_as, Executor, SqlitePool};
use uuid::Uuid;

use crate::repo::Repo;

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: RoleType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RoleType {
    SuperUser,
    ReadOnly,
    FullAccess,
    UpdateOnly,
}

impl RoleType {
    pub fn to_string(&self) -> String {
        match self {
            RoleType::SuperUser => "super_user".to_string(),
            RoleType::ReadOnly => "read_only".to_string(),
            RoleType::FullAccess => "full_access".to_string(),
            RoleType::UpdateOnly => "update_only".to_string(),
        }
    }

    pub fn from_string(role: &str) -> RoleType {
        match role {
            "super_user" => RoleType::SuperUser,
            "read_only" => RoleType::ReadOnly,
            "full_access" => RoleType::FullAccess,
            "update_only" => RoleType::UpdateOnly,
            _ => RoleType::ReadOnly,
        }
    }
}

#[derive(Clone, Debug, FromRow, PartialEq)]
pub struct UserRawData {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

impl User {
    pub async fn migrate(conn: &SqlitePool) -> Result<()> {
        conn.execute(USER_MIGRATION).await?;
        Ok(())
    }
    pub fn new(username: String, password: String, role: &str) -> Result<Self> {
        ok!(Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash: hash(password, 4)?,
            role: RoleType::from_string(role),
        })
    }

    pub async fn insert_db(self, conn: &SqlitePool) -> Result<Self> {
        query("insert into users values (?,  ?, ?, ?)")
            .bind(&self.id)
            .bind(&self.username)
            .bind(&self.password_hash)
            .bind(&self.role.to_string())
            .execute(conn)
            .await?;
        Ok(self)
    }

    pub async fn get_all(conn: &SqlitePool) -> Result<Vec<Self>> {
        let rows = query_as::<_, UserRawData>("select * from users")
            .fetch_all(conn)
            .await?;
        let users: Vec<User> = rows
            .into_iter()
            .map(|u| User {
                id: u.id,
                username: u.username,
                password_hash: u.password_hash,
                role: RoleType::from_string(&u.role),
            })
            .collect();
        Ok(users)
    }

    pub async fn get_by_id(id: &str, conn: &SqlitePool) -> Result<Self> {
        let row = query_as::<_, UserRawData>("select * from users where id = ?")
            .bind(id)
            .fetch_one(conn)
            .await?;
        let user = User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            role: RoleType::from_string(&row.role),
        };
        Ok(user)
    }

    pub async fn get_by_username(username: &str, conn: &SqlitePool) -> Result<Self> {
        let row = query_as::<_, UserRawData>("select * from users where username = ?")
            .bind(username)
            .fetch_one(conn)
            .await?;
        let user = User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            role: RoleType::from_string(&row.role),
        };
        Ok(user)
    }

    pub async fn super_user_exists(conn: &SqlitePool) -> Result<bool> {
        let rows = query_as::<_, UserRawData>("select * from users")
            .fetch_all(conn)
            .await?;

        Ok(rows
            .into_iter()
            .any(|u| u.role == RoleType::SuperUser.to_string()))
    }
}

pub const USER_MIGRATION: &str = r#"
    create table if not exists users (
        id text primary key,
        username text not null,
        password_hash text not null,
        role text not null
    );
    "#;

#[tokio::test]
async fn user_repo() {
    let pool = Repo::new(":memory:", true).await.unwrap().pool;
    User::migrate(&pool).await.unwrap();

    let super_user = User::new(
        "super".to_string(),
        "pass".to_string(),
        RoleType::SuperUser.to_string().as_str(),
    )
    .unwrap();
    super_user.insert_db(&pool).await.unwrap();
    let just_user = User::new(
        "just".to_string(),
        "pass".to_string(),
        RoleType::ReadOnly.to_string().as_str(),
    )
    .unwrap();
    just_user.insert_db(&pool).await.unwrap();
    let super_user_exists = User::super_user_exists(&pool).await.unwrap();
    assert!(super_user_exists);
    let users = User::get_all(&pool).await.unwrap();
    assert_eq!(users.len(), 2);
    let super_user = User::get_by_username("super", &pool).await.unwrap();
    assert!(super_user.role == RoleType::SuperUser);
    let just_user = User::get_by_username("just", &pool).await.unwrap();
    assert!(!(just_user.role == RoleType::ReadOnly));
}
