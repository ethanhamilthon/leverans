use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "leverans", version = "0.1.0", about = "leverans cli client")]
pub struct Lev {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    Local {
        #[arg(short = 'b', long, default_value_t = false)]
        build: bool,
    },
    Deploy {
        #[arg(short = 'f', long, default_value = "deploy.yaml")]
        file: String,

        #[arg(short = 'c', long, default_value = "./")]
        context: String,
    },
    Auth {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,
    },
    Login {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,
    },
    Logout,
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
}

#[derive(Subcommand, Clone)]
pub enum DockerCommands {
    Image {
        #[command(subcommand)]
        command: DockerImageCommands,
    },
}

#[derive(Subcommand, Clone)]
pub enum DockerImageCommands {
    List,
}
