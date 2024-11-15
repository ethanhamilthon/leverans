use anyhow::Result;
use shared::ok;

use crate::{
    commands::{Commands, Lev},
    handlers::{
        auth_handle::{handle_auth, handle_logout, whoami},
        deploy_handle::new_handle_deploy,
        handle_local,
        new_handler::handle_new,
        plan_handle::handle_plan,
        secret_handle::{add_secrets, list_secrets},
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
        } => new_handle_deploy(file, context, build, filter, only).await,
        Commands::Auth { address } => handle_auth(address, false).await,
        Commands::Login { address } => handle_auth(address, true).await,
        Commands::Logout => handle_logout().await,
        Commands::Whoami => whoami().await,
        Commands::Version => {
            println!("0.1.0");
            ok!(())
        }
        Commands::Secret { command } => match command {
            crate::commands::SecretCommands::Ls => list_secrets().await,
            crate::commands::SecretCommands::Add { key, value } => add_secrets(key, value).await,
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
    }
}
