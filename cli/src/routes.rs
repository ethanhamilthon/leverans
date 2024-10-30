use anyhow::Result;
use shared::ok;

use crate::{
    commands::{Commands, DockerCommands, DockerImageCommands, Lev},
    handlers::{
        auth_handle::{handle_auth, handle_logout, whoami},
        deploy_handle::DeployHandle,
        handle_local,
        secret_handle::{add_secrets, list_secrets},
    },
};

pub async fn handle_routes(cli: Lev) -> Result<()> {
    match cli.command {
        Commands::Local { build } => handle_local(build).await,
        Commands::Deploy { file, context } => DeployHandle::new(file, context)?.handle().await,
        Commands::Auth { address } => handle_auth(address, false).await,
        Commands::Login { address } => handle_auth(address, true).await,
        Commands::Logout => handle_logout().await,
        Commands::Whoami => whoami().await,
        Commands::Docker { command } => match command {
            DockerCommands::Image { command } => match command {
                DockerImageCommands::List => unimplemented!(),
            },
        },
        Commands::Version => {
            println!("0.1.0");
            ok!(())
        }
        Commands::Secret { command } => match command {
            crate::commands::SecretCommands::Ls => list_secrets().await,
            crate::commands::SecretCommands::Add { key, value } => add_secrets(key, value).await,
        },
    }
}
