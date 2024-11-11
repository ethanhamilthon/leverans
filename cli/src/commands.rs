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

        #[arg(short, long, default_value = None)]
        build: Option<Vec<String>>,

        filter: Option<String>,

        #[arg(short, long, default_value = None)]
        only: Option<Vec<String>>,
    },
    Version,
    Auth {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,
    },
    Login {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,
    },
    Logout,
    Whoami,
    Secret {
        #[command(subcommand)]
        command: SecretCommands,
    },
    Plan {
        #[arg(short = 'f', long, default_value = "deploy.yaml")]
        file: String,

        #[arg(short = 'c', long, default_value = "./")]
        context: String,

        #[arg(short, long, default_value = None)]
        build: Option<Vec<String>>,

        single_filter: Option<String>,

        #[arg(short, long, default_value = None)]
        only: Option<Vec<String>>,
    },
}

#[derive(Subcommand, Clone)]
pub enum SecretCommands {
    Ls,
    Add {
        #[arg(short = 'k', long, default_value = None)]
        key: Option<String>,
        #[arg(short = 'v', long, default_value = None)]
        value: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum DockerImageCommands {
    List,
}
