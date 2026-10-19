extern crate core;

mod args;
mod config;
mod database;

use crate::args::{parse_args};
use crate::database::Database;
use crate::config::Config;

fn main() -> Result<(), String> {
    let args = parse_args();
    let config = Config::from_args(&args).unwrap();

    // println!("Args: {:?}", args);
    // println!("Config: {:?}", config);

    let database = Database::new(args, config);



    database.run()
}
