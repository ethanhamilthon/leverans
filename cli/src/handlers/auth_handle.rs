use std::{
    io::{stdin, stdout, Write},
    sync::Arc,
};

use url::Url;

fn parse_url(input: &str) -> Result<Url> {
    // Если схема не указана, добавляем "http://"
    let input_with_scheme = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else {
        format!("http://{}", input)
    };

    // Пробуем разобрать URL
    Url::parse(&input_with_scheme).map_err(|e| e.into())
}

use anyhow::{anyhow, Result};
use shared::{console::ask, err, ok};

use crate::{api::API, data::UserData};

pub async fn handle_auth(
    init_address: Option<String>,
    init_username: Option<String>,
    init_password: Option<String>,
    is_login: bool,
    skip_confirm: bool,
) -> Result<()> {
    let url = if init_address.is_some() {
        parse_url(init_address.unwrap().as_str())?
    } else {
        let address = ask("Server Address: ")?;
        parse_url(address.as_str())?
    };
    let db = UserData::load_db(false).await?;
    if let Ok(user) = db.load_current_user().await {
        err!(anyhow!(
            "You are already logged for domain: {}",
            user.remote_url
        ))
    }
    let api = Arc::new(
        API::new(&url.to_string()).map_err(|e| anyhow!("⚠️  Error on parsing address: {}", e))?,
    );

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

    let username = if init_username.is_some() {
        init_username.unwrap()
    } else {
        ask("Username: ")?
    };
    let password = if init_password.is_some() {
        init_password.unwrap()
    } else {
        ask("Password: ")?
    };
    if !skip_confirm {
        let confirm = ask(&format!(
            "Address: {} | Username: {} | Password: {} | Confirm (y/n): ",
            url,
            username,
            "*".repeat(password.len())
        ))?;

        if confirm != "y" {
            err!(anyhow!("💨 Aborted, no changes were made"));
        }
    }
    if is_login {
        let token = api
            .login_user(&username, &password)
            .await
            .map_err(|e| anyhow!("Error on login user: {}", e))?;

        if token.is_empty() {
            err!(anyhow!("Error on loginning user: Empty token"));
        }

        db.save_user(token, url.to_string(), username).await?;

        println!("\n✔︎ User logged in uccessfully, you are in system.\n Use `lev new` or `lev init` in root of your project to get started.");
    } else {
        let token = api
            .register_super_user(&username, &password)
            .await
            .map_err(|e| anyhow!("Error on registering super user: {}", e))?;

        if token.is_empty() {
            err!(anyhow!("Error on registering super user: Empty token"));
        }

        db.save_user(token, url.to_string(), username).await?;

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
    println!(
        "✔︎ Current user: IP: {}  |  Username: {}",
        user.remote_url, user.username
    );
    Ok(())
}
