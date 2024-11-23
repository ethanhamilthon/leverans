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

        #[arg(short = 's', long, default_value_t = false)]
        skip_confirm: bool,

        #[arg(short = 'u', long, default_value_t = false)]
        unfold: bool,

        #[arg(short = 't', long, default_value = None)]
        timeout: Option<u64>,
    },
    Rollback {
        #[arg(short = 'f', long, default_value = "deploy.yaml")]
        file: String,

        #[arg(short = 'c', long, default_value = "./")]
        context: String,

        #[arg(short = 's', long, default_value_t = false)]
        skip_confirm: bool,

        #[arg(short = 'u', long, default_value_t = false)]
        unfold: bool,

        #[arg(short = 't', long, default_value = None)]
        timeout: Option<u64>,
    },
    Version,
    Auth {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,

        #[arg(short = 'u', long, help = "your username", default_value = None)]
        username: Option<String>,

        #[arg(short = 'p', long, help = "your password", default_value = None)]
        password: Option<String>,

        #[arg(short = 's', long, default_value_t = false)]
        skip_confirm: bool,
    },
    Login {
        #[arg(short = 'a', long, help = "the address of your server, eg 312.89.06.172 or mydomain.com", default_value = None)]
        address: Option<String>,

        #[arg(short = 'u', long, help = "your username", default_value = None)]
        username: Option<String>,

        #[arg(short = 'p', long, help = "your password", default_value = None)]
        password: Option<String>,

        #[arg(short = 's', long, default_value_t = false)]
        skip_confirm: bool,
    },
    User {
        #[command(subcommand)]
        com: UserCommands,
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

        #[arg(short = 'u', long, default_value_t = false)]
        unfold: bool,
    },
    New {
        name: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum UserCommands {
    Ls,
    Create {
        #[arg(short = 'u', long, help = "username of new user", default_value = None)]
        username: Option<String>,

        #[arg(short = 'p', long, help = "password of new user", default_value = None)]
        password: Option<String>,

        #[arg(short = 'r', long, help = "role of new user", default_value = None)]
        role: Option<String>,

        #[arg(short = 's', long, default_value_t = false)]
        skip_confirm: bool,
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
    Update {
        #[arg(short = 'k', long, default_value = None)]
        key: Option<String>,
        #[arg(short = 'v', long, default_value = None)]
        value: Option<String>,
    },
    Delete {
        key: Option<String>,
    },
    Show {
        key: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum DockerImageCommands {
    List,
}
