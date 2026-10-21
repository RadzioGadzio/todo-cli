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
        parent_id: Option<u32>,
    },
    EDIT {
        #[arg(required = true)]
        id: u32,
        #[arg(short, long)]
        title: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        parent_id: Option<u32>,
    },
    REMOVE {
        #[arg(help = "Todo ids to remove. If omitted removes all done todos")]
        ids: Vec<u32>,

        #[arg(short, long, help = "Remove all children", default_value = "false")]
        recursive: bool,

        #[arg(short, long, help = "Remove all", default_value = "false")]
        all: bool,

    },
    DONE {
        #[arg(required = true, help = "Todo ids to mark done")]
        ids: Vec<u32>,

        #[arg(short, long, help = "Mark all children as done", default_value = "false")]
        recursive: bool,

        #[arg(short, long, help = "Mark item as not checked", default_value = "false")]
        inverted: bool
    },
    LIST,
    RESET,
    BACKUP {
        #[arg(short, long, help = "Index of backup to restore")]
        index: Option<u32>,
        #[arg(short, long, help = "Backup path to restore")]
        path: Option<PathBuf>,
        
        #[arg(short, long, help = "Create new backup")]
        create: bool,
        
        #[arg(short, long, help = "New backup name")]
        name: Option<String>,
    }
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