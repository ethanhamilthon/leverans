use anyhow::Result;
use shared::ok;

use crate::{
    commands::{Commands, Lev, UserCommands},
    handlers::{
        auth_handle::{create_user, handle_auth, handle_logout, whoami},
        deploy_handle::new_handle_deploy,
        handle_local,
        new_handler::handle_new,
        plan_handle::handle_plan,
        secret_handle::{add_secrets, delete_secrets, list_secrets, update_secrets},
    },
};

pub async fn handle_routes(cli: Lev) -> Result<()> {
    match cli.command {
        Commands::Local { build } => handle_local(build).await,
        Commands::Deploy {
            context,
            build,
            filter,
            only,
            file,
            skip_confirm,
        } => new_handle_deploy(file, context, build, filter, only, skip_confirm).await,
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
            println!("0.1.0");
            ok!(())
        }
        Commands::Secret { command } => match command {
            crate::commands::SecretCommands::Ls => list_secrets().await,
            crate::commands::SecretCommands::Add { key, value } => add_secrets(key, value).await,
            crate::commands::SecretCommands::Update { key, value } => {
                update_secrets(key, value).await
            }
            crate::commands::SecretCommands::Delete { key } => delete_secrets(key).await,
        },
        Commands::Plan {
            file,
            context,
            build,
            single_filter,
            only,
        } => {
            handle_plan(single_filter, only, file, context, build).await?;
            ok!(())
        }
        Commands::New { name } => handle_new(name),
        Commands::User { com } => match com {
            UserCommands::List => todo!(),
            UserCommands::Create {
                username,
                password,
                role,
                skip_confirm,
            } => create_user(username, password, role, skip_confirm).await,
        },
    }
}
