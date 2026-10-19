mod executor;
mod item;

use std::{fs, io};
use std::path::PathBuf;
use crate::args::Args;
use crate::config::Config;
use crate::database::executor::execute_action;

pub use crate::database::item::Item;

pub struct Database {
    config: Config,
    args: Args,
}

impl Database {
    pub fn new(args: Args, config: Config) -> Database {
        Database {
            args,
            config
        }
    }
    pub fn run(&self) -> Result<(), String> {
        let mut items = self.read()?;
        let changed = execute_action(&mut items, &self.args.action)?;

        if changed {
            self.write(&items)?;
        }

        Ok(())
    }
    

    fn read(&self) -> Result<Vec<Item>, String> {
        if !self.config.db_path.exists() {
            return Ok(Vec::new());
        }

        let opened = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(&self.config.db_path);

        let mut reader = match opened {
            Ok(r) => r,
            Err(e) if matches!(e.kind(), csv::ErrorKind::Io(io) if io.kind() == io::ErrorKind::NotFound) => {
                return Ok(Vec::new())
            }
            Err(e) => { return Err(format!("Cannot read database file: {}", e)) }
        };
        let headers = match reader.headers() {
            Ok(h) => h,
            Err(e) => { return Err(format!("Cannot read database headers: {}", e)) }
        };
        

        if !headers.iter().all(|h| Item::HEADERS.contains(&h)) {
            let found: Vec<&str> = headers.iter().collect();
            return Err(format!("Invalid headers. Expected: {:?}, found: {:?}", Item::HEADERS, found));
        }

        Ok(match reader
            .deserialize()
            .collect::<Result<Vec<Item>, _>>() {
                Ok(items) => items,
                Err(e) => { return Err(format!("Cannot read database: {}", e)) }
        })
    }

    fn write(&self, items: &Vec<Item>) -> Result<(), String> {
        fn _write(items: &Vec<Item>, tmp: &PathBuf) -> Result<(), io::Error> {
            let mut writer = csv::WriterBuilder::new()
                .delimiter(b';')
                .has_headers(true)
                .quote_style(csv::QuoteStyle::Always)
                .from_path(&tmp)?;
            // writer.write_record(Item::HEADERS)?;
            for item in items {
                writer.serialize(item)?;
            }
            writer.flush()?;
            Ok(())
        }
        
        let db_path = &self.config.db_path;
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let tmp = &db_path.with_extension("csv.tmp");
        
        _write(items, tmp).map_err(|err| err.to_string())?;
        
        fs::rename(tmp, db_path).map_err(|err| err.to_string())?;
        
        Ok(())
    }
}