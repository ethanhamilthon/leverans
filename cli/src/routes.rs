use anyhow::Result;
use shared::ok;

use crate::{
    commands::{Commands, Lev, UserCommands},
    handlers::{
        auth_handle::{create_user, handle_auth, handle_logout, list_user, whoami},
        deploy_handle::new_handle_deploy,
        handle_local,
        new_handler::handle_new,
        plan_handle::handle_plan,
        secret_handle::{add_secrets, delete_secrets, list_secrets, show_secret, update_secrets},
    },
};

pub async fn handle_routes(cli: Lev) -> Result<()> {
    match cli.command {
        Commands::Deploy {
            context,
            build,
            filter,
            only,
            file,
            skip_confirm,
            unfold,
            timeout,
        } => {
            new_handle_deploy(
                file,
                context,
                build,
                filter,
                only,
                skip_confirm,
                unfold,
                false,
                timeout,
            )
            .await
        }
        Commands::Auth {
            address,
            password,
            username,
            skip_confirm,
        } => handle_auth(address, username, password, false, skip_confirm).await,
        Commands::Login {
            address,
            password,
            username,
            skip_confirm,
        } => handle_auth(address, username, password, true, skip_confirm).await,
        Commands::Logout => handle_logout().await,
        Commands::Whoami => whoami().await,
        Commands::Version => {
            let version = option_env!("LEV_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
            println!("{}", version);
            ok!(())
        }
        Commands::Secret { command } => match command {
            crate::commands::SecretCommands::Ls => list_secrets().await,
            crate::commands::SecretCommands::Add { key, value } => add_secrets(key, value).await,
            crate::commands::SecretCommands::Update { key, value } => {
                update_secrets(key, value).await
            }
            crate::commands::SecretCommands::Delete { key } => delete_secrets(key).await,
            crate::commands::SecretCommands::Show { key } => show_secret(key).await,
        },
        Commands::Plan {
            file,
            context,
            build,
            single_filter,
            only,
            unfold,
        } => {
            handle_plan(single_filter, only, file, context, build, unfold, false).await?;
            ok!(())
        }
        Commands::New { name } => handle_new(name),
        Commands::User { com } => match com {
            UserCommands::Ls => list_user().await,
            UserCommands::Create {
                username,
                password,
                role,
                skip_confirm,
            } => create_user(username, password, role, skip_confirm).await,
        },
        Commands::Rollback {
            file,
            context,
            skip_confirm,
            unfold,
            timeout,
        } => {
            new_handle_deploy(
                file,
                context,
                None,
                None,
                None,
                skip_confirm,
                unfold,
                true,
                timeout,
            )
            .await
        }
    }
}
