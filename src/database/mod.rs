mod executor;
mod item;

use std::{fs, io};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use chrono::offset::Utc;
use chrono::DateTime;

use crate::args::{Action, Args};
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
        let items_result = self.read(None);
        if let Err(e) = items_result {
            return Err(e);
        }
        let mut items = items_result?;

        if let Action::BACKUP { .. } = &self.args.action {
            if self.backup_action(&self.args.action, &mut items)? {
                // Restore backup
                self.write(&items, None)?;
            }
            return Ok(());
        }

        if execute_action(&mut items, &self.args.action)? {
            if let Err(e) = self.backup(None) {
                return Err(e);
            }
            self.write(&items, None)?;
        }

        Ok(())
    }

    fn backup_action(&self, action: &Action, items: &mut Vec<Item>) -> Result<bool, String> {
        match action {
            Action::BACKUP { index, path } => {
                if let Some(index) = index {
                    // Restore backup from index
                    todo!()
                }
                else if let Some(path) = path {
                    // Restore backup from path
                    todo!()
                }
                // List backups
                else {
                    self.print_backups()?;
                }
            }
            _ => {}
        }

        Ok(false)
    }

    fn read(&self, path: Option<&PathBuf>) -> Result<Vec<Item>, String> {
        let path = match path {
            Some(path) => path,
            None => &self.config.db_path
        };
        if !path.exists() {
            return Ok(Vec::new());
        }

        let opened = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(path);

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
    fn write(&self, items: &Vec<Item>, path: Option<&PathBuf>) -> Result<(), String> {
        fn _write(items: &Vec<Item>, tmp: &PathBuf) -> Result<(), io::Error> {
            let mut writer = csv::WriterBuilder::new()
                .delimiter(b';')
                .has_headers(true)
                .quote_style(csv::QuoteStyle::Always)
                .from_path(&tmp)?;
            for item in items {
                writer.serialize(item)?;
            }
            writer.flush()?;
            Ok(())
        }

        let path = match path {
            Some(p) => p,
            None => &self.config.db_path
        };


        if let Some(ext) = path.extension() {
            if ext.is_empty() || ext != "csv" {
                return Err(format!("Invalid database extension: {}. Should be '.csv'", path.display()));
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }

        let tmp = &path.with_extension("csv.tmp");

        _write(items, &tmp).map_err(|err| err.to_string())?;
        
        fs::rename(tmp, path).map_err(|err| err.to_string())?;
        
        Ok(())
    }
    fn backup(&self, name: Option<&str>) -> Result<(), String> {
        let now: DateTime<Utc> = SystemTime::now().into();
        let backup_dir = &self.config.backups_dir;

        if !backup_dir.exists() || !backup_dir.is_dir() {
            fs::create_dir_all(backup_dir).map_err(|err| err.to_string())?;
        }

        let items = self.read(None)?;
        let mut file_name = now.format("%F_%H-%M-%S").to_string();
        if let Some(name) = name {
            file_name.push_str("_");
            file_name.push_str(name);
        }
        let path = backup_dir.join(PathBuf::from(format!("{}.csv", file_name)));

        self.write(&items, Some(&path))?;

        self.prune_backups()?;

        Ok(())
    }

    /** Collect (path, modified time) for every backup file. */
    pub fn collect_backups(&self) -> Result<Vec<(PathBuf, SystemTime)>, String> {
        let mut backups: Vec<(PathBuf, SystemTime)> = Vec::new();
        let entries = match fs::read_dir(&self.config.backups_dir) {
            Ok(entries) => entries,
            Err(e) => return Err(format!("Cannot read backups directory: {}", e)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => return Err(format!("Cannot read backups directory entry: {}", e)),
            };
            let path = entry.path();

            // Skips *.csv.tmp leftovers and anything foreign.
            match path.extension() {
                Some(ext) if ext == "csv" => {}
                _ => continue,
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => return Err(format!("Cannot read metadata of {}: {}", path.display(), e)),
            };
            if !metadata.is_file() {
                continue;
            }
            let modified = match metadata.modified() {
                Ok(t) => t,
                Err(e) => return Err(format!("Cannot read modified time of {}: {}", path.display(), e)),
            };

            backups.push((path, modified));
        }

        // Newest first
        backups.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(backups)
    }
    pub fn prune_backups(&self) -> Result<(), String> {
        let max_count = self.config.max_backups;
        let max_days = self.config.max_backup_age_days;
        let max_age = Duration::from_secs(max_days * 24 * 60 * 60);

        let backups: Vec<(PathBuf, SystemTime)> = self.collect_backups()?;

        let now = SystemTime::now();
        for (index, (path, modified)) in backups.iter().enumerate() {
            let age = now.duration_since(*modified).unwrap_or_else(|_| Duration::ZERO);

            let too_many = max_count > 0 && index >= max_count;
            let too_old = max_days > 0 && age > max_age;

            if too_many || too_old {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Cannot delete backup {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }
    pub fn print_backups(&self) -> Result<(), String> {
        let backups: Vec<(PathBuf, SystemTime)> = self.collect_backups()?;

        for (index, (path, time)) in backups.iter().enumerate() {
            let time: DateTime<Utc> = time.clone().into();
            println!("{} | {} [{}]",
                     index,
                     path.display(),
                     time.format("%F %H:%M:%S")
            );
        }

        Ok(())
    }
}