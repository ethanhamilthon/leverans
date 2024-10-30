use anyhow::Result;

use crate::{
    commands::{Commands, DockerCommands, DockerImageCommands, Lev},
    handlers::{
        auth_handle::{handle_auth, handle_logout},
        deploy_handle::DeployHandle,
        handle_local,
    },
};

pub async fn handle_routes(cli: Lev) -> Result<()> {
    match cli.command {
        Commands::Local { build } => handle_local(build).await,
        Commands::Deploy { file, context } => DeployHandle::new(file, context)?.handle().await,
        Commands::Auth { address } => handle_auth(address, false).await,
        Commands::Login { address } => handle_auth(address, true).await,
        Commands::Logout => handle_logout().await,
        Commands::Docker { command } => match command {
            DockerCommands::Image { command } => match command {
                DockerImageCommands::List => unimplemented!(),
            },
        },
    }
}
