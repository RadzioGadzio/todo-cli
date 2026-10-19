use std::path::{PathBuf};
use clap::{Parser, Subcommand};


#[derive(Debug, Subcommand)]
pub enum Action {
    ADD {
        #[arg(required = true, help = "Todo title")]
        title: String,
        #[arg(short, long, required = false,  help = "Todo description", value_name = "TEXT")]
        description: Option<String>,
        #[arg(short, long, required = false, help = "Parent todo item", value_name = "ID")]
        parent: Option<u32>,
    },
    EDIT {
        #[arg(required = true)]
        id: u32,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        parent: Option<u32>,
    },
    REMOVE {
        #[arg(required = true, help = "Todo ids to remove")]
        id: Vec<String>
    },
    DONE {
        #[arg(required = true, help = "Todo ids to mark done")]
        id: Vec<String>,
    },
    LIST {
        #[arg(long, short, help = "Show description")]
        description: bool,
    },
    RESET,
    BACKUP
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,

    #[arg(short, long, default_value = "./todos.yaml")]
    pub config: PathBuf,
}

pub fn parse_args() -> Args {
    Args::parse()
}