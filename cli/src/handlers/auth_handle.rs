use std::{
    io::{stdin, stdout, Write},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use shared::{err, ok};

use crate::{api::API, data::UserData};

pub async fn handle_auth(address: Option<String>, is_login: bool) -> Result<()> {
    let url = address.ok_or(anyhow!("No address provided"))?;
    let db = UserData::load_db(false).await?;
    if let Ok(user) = db.load_current_user().await {
        err!(anyhow!(
            "You are already logged for domain: {}",
            user.remote_url
        ))
    }
    let api = Arc::new(API::new(&url).map_err(|e| anyhow!("⚠️  Error on parsing address: {}", e))?);

    if is_login {
        println!(
            "👋 Welcome back to Leverans! Trying to login with account to: {}\n",
            url
        );
    } else {
        println!(
            "👋 Welcome to Leverans! Trying to register new account to: {}\n",
            url
        );
    }

    let h_api = api.clone();
    let health_task = tokio::task::spawn(async move { h_api.health_check().await });
    let s_api = api.clone();
    let super_user_task = tokio::task::spawn(async move { s_api.is_super_user_exists().await });

    let (health, super_user) = tokio::join!(health_task, super_user_task);
    let _ = health?.map_err(|e| anyhow!("Error on health check: {}", e))?;
    let super_user_exists =
        super_user?.map_err(|e| anyhow!("Error on getting super user:{}", e))?;
    match (super_user_exists, is_login) {
        (false, true) => {
            err!(anyhow!(
                "There is no user in the system, use 'lev auth' instead"
            ));
        }
        (true, false) => {
            err!(anyhow!(
                "There is already a super user, use 'lev login' instead"
            ));
        }
        (_, _) => {}
    }

    if is_login {
        println!("✔︎ Ready to login user.\n");
    } else {
        println!("✔︎ Ready to create a super user.\n");
    }
    stdout().flush().unwrap();

    print!("Username: ");
    stdout().flush()?;
    let mut username = String::new();
    stdin()
        .read_line(&mut username)
        .map_err(|e| anyhow!("Error on reading username: {}", e))?;
    username = username.trim().to_string();

    let mut password = String::new();
    print!("Password: ");
    stdout().flush()?;
    stdin()
        .read_line(&mut password)
        .map_err(|e| anyhow!("Error on reading password: {}", e))?;
    password = password.trim().to_string();

    let mut confirm = String::new();
    print!(
        "Username: {}, Password: {}. Please confirm (y/n): ",
        username,
        "*".repeat(password.len()).as_str()
    );
    stdout().flush()?;
    stdin()
        .read_line(&mut confirm)
        .map_err(|e| anyhow!("Error on reading confirmation {}", e))?;
    confirm = confirm.trim().to_string();

    if confirm != "y" {
        err!(anyhow!("💨 Aborted, no changes were made"));
    }
    if is_login {
        let token = api
            .login_user(&username, &password)
            .await
            .map_err(|e| anyhow!("Error on login user: {}", e))?;

        if token.is_empty() {
            err!(anyhow!("Error on loginning user: Empty token"));
        }

        db.save_user(token, url).await?;

        println!("\n✔︎ User logged in uccessfully, you are in system.\n Use `lev new` or `lev init` in root of your project to get started.");
    } else {
        let token = api
            .register_super_user(&username, &password)
            .await
            .map_err(|e| anyhow!("Error on registering super user: {}", e))?;

        if token.is_empty() {
            err!(anyhow!("Error on registering super user: Empty token"));
        }

        db.save_user(token, url).await?;

        println!("\n✔︎ Super user created successfully, you are in system.\n Use `lev new` or `lev init` in root of your project to get started.");
    }
    ok!(())
}

pub async fn handle_logout() -> Result<()> {
    let db = UserData::load_db(false).await?;
    let user = db.load_current_user().await?;

    db.delete_user(user.id).await?;
    println!("✔︎ User logged out successfully");
    Ok(())
}

pub async fn whoami() -> Result<()> {
    let db = UserData::load_db(false).await?;
    let user = db.load_current_user().await?;
    println!("✔︎ Current user url: {}", user.remote_url);
    Ok(())
}
