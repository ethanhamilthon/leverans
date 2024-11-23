use anyhow::{anyhow, Result};
use shared::{err, ok};

use crate::{api::API, data::UserData};

pub async fn add_secrets(key: Option<String>, value: Option<String>) -> Result<()> {
    let secret_key = match key {
        Some(key) => key,
        None => err!(anyhow!("secret key is required")),
    };
    let secret_value = match value {
        Some(value) => value,
        None => err!(anyhow!("secret value is required")),
    };

    let user = UserData::load_db(false).await?.load_current_user().await?;
    API::new(&user.remote_url)?
        .add_secret(&secret_key, &secret_value, &user.remote_token)
        .await?;

    println!("✔︎ Secret added successfully");
    ok!(())
}

pub async fn list_secrets() -> Result<()> {
    let user = UserData::load_db(false).await?.load_current_user().await?;

    let secrets = API::new(&user.remote_url)?
        .list_secret(&user.remote_token)
        .await?;

    println!("Found {} secrets: \n", secrets.len());

    for secret in secrets {
        println!("Key: {}  |  Created at: {}", secret.key, secret.created_at);
    }

    ok!(())
}

pub async fn update_secrets(key: Option<String>, value: Option<String>) -> Result<()> {
    let secret_key = match key {
        Some(key) => key,
        None => err!(anyhow!("secret key is required")),
    };
    let secret_value = match value {
        Some(value) => value,
        None => err!(anyhow!("secret value is required")),
    };

    let user = UserData::load_db(false).await?.load_current_user().await?;
    API::new(&user.remote_url)?
        .update_secret(&secret_key, &secret_value, &user.remote_token)
        .await?;

    println!("✔︎ Secret updated successfully");
    ok!(())
}

pub async fn delete_secrets(key: Option<String>) -> Result<()> {
    let secret_key = match key {
        Some(key) => key,
        None => err!(anyhow!("secret key is required")),
    };

    let user = UserData::load_db(false).await?.load_current_user().await?;
    API::new(&user.remote_url)?
        .delete_secret(&secret_key, &user.remote_token)
        .await?;

    println!("✔︎ Secret deleted successfully");
    ok!(())
}

pub async fn show_secret(key: Option<String>) -> Result<()> {
    let secret_key = match key {
        Some(key) => key,
        None => err!(anyhow!("secret key is required")),
    };
    let user = UserData::load_db(false).await?.load_current_user().await?;
    let secret_value = API::new(&user.remote_url)?
        .show_secret(&secret_key, &user.remote_token)
        .await?;

    println!("✔︎  {} : {} ", secret_key, secret_value);
    ok!(())
}
