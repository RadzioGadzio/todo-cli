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
        if let Action::Backup { .. } = &self.args.action {
            return self.backup_action(&self.args.action);
        }
        let mut items = self.read(None)?;


        if execute_action(&mut items, &self.args.action)? {
            self.backup(None)?;
            self.write(&items, None)?;
        }

        Ok(())
    }

    fn backup_action(&self, action: &Action) -> Result<(), String> {
        if let Action::Backup { index, path, name, create } = action {
            if *create {
                let name =  match name {
                    Some(name) => name.as_str(),
                    None => "manual"
                };
                self.backup(Some(name))?;
                println!("Backup '{}' created", name);
                return Ok(());
            }

            let target: Option<PathBuf> = match (path, index) {
                (Some(p), _) => Some(p.clone()),
                (None, Some(i)) => Some(self.get_backup_path(*i)?),
                (None, None) => None
            };

            match target {
                // Restore backup from path
                Some(p) => self.restore(&p)?,
                // List backups
                None => self.print_backups()?
            }
        }

        Ok(())
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
                .from_path(tmp)?;
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


        if let Some(ext) = path.extension() &&
            (ext.is_empty() || ext != "csv") {
            return Err(format!("Invalid database extension: {}. Should be '.csv'", path.display()));
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }

        let tmp = &path.with_extension("csv.tmp");

        _write(items, tmp).map_err(|err| err.to_string())?;
        
        fs::rename(tmp, path).map_err(|err| err.to_string())?;
        
        Ok(())
    }

    pub fn backup(&self, name: Option<&str>) -> Result<(), String> {
        if !&self.config.db_path.exists() {
            eprintln!("Couldn't find database: {}", self.config.db_path.display());

            return Ok(());
        }
        let now: DateTime<Utc> = SystemTime::now().into();
        let backup_dir = &self.config.backups_dir;

        if !backup_dir.exists() || !backup_dir.is_dir() {
            fs::create_dir_all(backup_dir).map_err(|err| err.to_string())?;
        }

        let mut file_name = now.format("%F_%H-%M-%S").to_string();
        if let Some(name) = name {
            file_name.push("_".chars().next_back().unwrap());
            file_name.push_str(name);
        }
        let path = backup_dir.join(PathBuf::from(format!("{}.csv", file_name)));
        if let Err(e) = fs::copy(&self.config.db_path, &path) {
            return Err(format!("Cannot backup database: {}", e));
        }
        self.prune_backups()?;

        Ok(())
    }
    pub fn get_backup_path(&self, index: u32) -> Result<PathBuf, String> {
        let backups = self.collect_backups()?;
        match backups.get(index as usize) {
            Some((path, _)) => Ok(path.clone()),
            None => Err(format!(
                "Backup index {} out of range, {} backup(s) found",
                index,
                backups.len()
            )),
        }
    }

    pub fn restore(&self, path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if let Err(e) = fs::copy(path, &self.config.db_path) {
            return Err(format!("Cannot restore database: {}", e));
        }
        println!("Backup successfully restored: {}", path.display());
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
        backups.sort_by_key(|a| std::cmp::Reverse(a.1));

        Ok(backups)
    }
    pub fn prune_backups(&self) -> Result<(), String> {
        let max_count = self.config.max_backups;
        let max_days = self.config.max_backup_age_days;
        let max_age = Duration::from_secs(max_days * 24 * 60 * 60);

        let backups: Vec<(PathBuf, SystemTime)> = self.collect_backups()?;

        let now = SystemTime::now();
        for (index, (path, modified)) in backups.iter().enumerate() {
            let age = now.duration_since(*modified).unwrap_or(Duration::ZERO);

            let too_many = max_count > 0 && index >= max_count;
            let too_old = max_days > 0 && age > max_age;

            if (too_many || too_old) && let Err(e) = fs::remove_file(path) {
                eprintln!("Cannot delete backup {}: {}", path.display(), e);
            }
        }

        Ok(())
    }
    pub fn print_backups(&self) -> Result<(), String> {
        let backups: Vec<(PathBuf, SystemTime)> = self.collect_backups()?;

        for (index, (path, time)) in backups.iter().enumerate() {
            let time: DateTime<Utc> = (*time).into();
            println!("{} | {} [{}]",
                     index,
                     path.display(),
                     time.format("%F %H:%M:%S")
            );
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{self, SetupItems, TempDir};

    

    fn db_with(dir: &TempDir, action: Action, max_backups: usize, max_days: u64) -> Database {
        let config = Config {
            db_path: dir.path().join("todos.csv"),
            backups_dir: dir.path().join("backups"),
            max_backups,
            max_backup_age_days: max_days,
        };
        Database::new(Args { action, config: PathBuf::from(".tmp/config.yaml") }, config)
    }

    fn list_action() -> Action {
        Action::Backup { index: None, path: None, name: None, create: false }
    }

    fn db(dir: &TempDir) -> Database {
        db_with(dir, list_action(), 0, 0)
    }

    /// Creates a backup file with the given modification time offset.
    fn make_backup(database: &Database, name: &str, content: &str, secs_ago: u64) -> PathBuf {
        fs::create_dir_all(&database.config.backups_dir).unwrap();
        let path = database.config.backups_dir.join(name);
        fs::write(&path, content).unwrap();
        let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(secs_ago)).unwrap();
        path
    }

    fn ids(items: &[Item]) -> Vec<u32> {
        items.iter().map(|i| i.id()).collect()
    }

    #[test]
    fn _new() {
        let dir = TempDir::new("new");
        let database = db(&dir);

        assert_eq!(database.config.db_path, dir.path().join("todos.csv"));
        assert_eq!(database.config.backups_dir, dir.path().join("backups"));
        assert!(matches!(database.args.action, Action::Backup { .. }));
    }

    #[test]
    fn _run() {
        let dir = TempDir::new("run");
        let action = Action::Backup { index: None, path: None, name: Some("run".to_string()), create: true };
        let database = db_with(&dir, action, 0, 0);
        database.write(&tests::setup_items(SetupItems::Flat), None).unwrap();

        assert!(database.run().is_ok(), "run() should succeed for Backup action");

        let backups = database.collect_backups().unwrap();
        assert_eq!(backups.len(), 1, "Expected one backup created through run()");
    }

    #[test]
    fn _backup_action() {
        let dir = TempDir::new("backup_action");
        let database = db(&dir);
        let items = tests::setup_items(SetupItems::Flat);
        database.write(&items, None).unwrap();

        // create
        let create = Action::Backup { index: None, path: None, name: Some("manual".to_string()), create: true };
        assert!(database.backup_action(&create).is_ok());
        let backups = database.collect_backups().unwrap();
        assert_eq!(backups.len(), 1, "Backup not created");
        assert!(backups[0].0.to_string_lossy().ends_with("_manual.csv"));

        // list
        assert!(database.backup_action(&list_action()).is_ok());

        // restore by index
        fs::remove_file(&database.config.db_path).unwrap();
        let restore_idx = Action::Backup { index: Some(0), path: None, name: None, create: false };
        assert!(database.backup_action(&restore_idx).is_ok());
        assert_eq!(ids(&database.read(None).unwrap()), ids(&items), "Restore by index failed");

        // restore by path
        fs::remove_file(&database.config.db_path).unwrap();
        let restore_path = Action::Backup { index: None, path: Some(backups[0].0.clone()), name: None, create: false };
        assert!(database.backup_action(&restore_path).is_ok());
        assert_eq!(ids(&database.read(None).unwrap()), ids(&items), "Restore by path failed");

        // index out of range
        let bad = Action::Backup { index: Some(99), path: None, name: None, create: false };
        assert!(database.backup_action(&bad).is_err(), "Out of range index should error");
    }

    #[test]
    fn _read() {
        let dir = TempDir::new("read");
        let database = db(&dir);

        // missing file
        assert!(database.read(None).unwrap().is_empty(), "Missing file should yield empty vector");

        // roundtrip
        let items = tests::setup_items(SetupItems::RecursiveDepth2);
        database.write(&items, None).unwrap();
        let read = database.read(None).unwrap();
        assert_eq!(ids(&read), ids(&items));

        // explicit path
        let other = dir.path().join("other.csv");
        database.write(&items, Some(&other)).unwrap();
        assert_eq!(ids(&database.read(Some(&other)).unwrap()), ids(&items));

        // invalid headers
        fs::write(&database.config.db_path, "foo;bar\n1;2\n").unwrap();
        assert!(database.read(None).is_err(), "Invalid headers should error");
    }

    #[test]
    fn _write() {
        let dir = TempDir::new("write");
        let database = db(&dir);
        let items = tests::setup_items(SetupItems::Flat);

        assert!(database.write(&items, None).is_ok());
        assert!(database.config.db_path.exists(), "Database file not created");
        assert!(!database.config.db_path.with_extension("csv.tmp").exists(), "Tmp file left behind");
        assert_eq!(ids(&database.read(None).unwrap()), ids(&items));

        // creates missing parent directories
        let nested = dir.path().join("a").join("b").join("db.csv");
        assert!(database.write(&items, Some(&nested)).is_ok());
        assert!(nested.exists(), "Parent directories not created");

        // invalid extension
        let bad = dir.path().join("db.txt");
        assert!(database.write(&items, Some(&bad)).is_err(), "Non-csv extension should error");
        assert!(!bad.exists());
    }

    #[test]
    fn _backup() {
        let dir = TempDir::new("backup");
        let database = db(&dir);

        // no database: Ok, nothing created
        assert!(database.backup(None).is_ok());
        assert!(!database.config.backups_dir.exists(), "Backup dir created without database");

        database.write(&tests::setup_items(SetupItems::Flat), None).unwrap();

        assert!(database.backup(Some("named")).is_ok());
        let backups = database.collect_backups().unwrap();
        assert_eq!(backups.len(), 1, "Expected one backup");
        assert!(backups[0].0.to_string_lossy().ends_with("_named.csv"), "Name suffix missing: {:?}", backups[0].0);
        assert_eq!(
            fs::read_to_string(&backups[0].0).unwrap(),
            fs::read_to_string(&database.config.db_path).unwrap(),
            "Backup content differs from database"
        );
    }

    #[test]
    fn _get_backup_path() {
        let dir = TempDir::new("get_backup_path");
        let database = db(&dir);
        let old = make_backup(&database, "old.csv", "", 200);
        let new = make_backup(&database, "new.csv", "", 10);

        assert_eq!(database.get_backup_path(0).unwrap(), new, "Index 0 should be the newest");
        assert_eq!(database.get_backup_path(1).unwrap(), old);
        assert!(database.get_backup_path(2).is_err(), "Out of range index should error");
    }

    #[test]
    fn _restore() {
        let dir = TempDir::new("restore");
        let database = db(&dir);
        let items = tests::setup_items(SetupItems::RecursiveDepth1);
        database.write(&items, None).unwrap();
        database.backup(Some("restore")).unwrap();
        let backup_path = database.collect_backups().unwrap().remove(0).0;

        database.write(&tests::setup_items(SetupItems::Flat), None).unwrap();
        assert!(database.restore(&backup_path).is_ok());
        assert_eq!(ids(&database.read(None).unwrap()), ids(&items), "Database not restored");

        let missing = dir.path().join("missing.csv");
        assert!(database.restore(&missing).is_err(), "Missing backup path should error");
    }

    #[test]
    fn _collect_backups() {
        let dir = TempDir::new("collect_backups");
        let database = db(&dir);

        // missing directory
        assert!(database.collect_backups().is_err(), "Missing backups dir should error");

        let old = make_backup(&database, "old.csv", "", 300);
        let new = make_backup(&database, "new.csv", "", 10);
        make_backup(&database, "leftover.csv.tmp", "", 5);
        make_backup(&database, "notes.txt", "", 5);
        fs::create_dir_all(database.config.backups_dir.join("dir.csv")).unwrap();

        let backups = database.collect_backups().unwrap();
        let paths: Vec<&PathBuf> = backups.iter().map(|(p, _)| p).collect();

        assert_eq!(paths, vec![&new, &old], "Expected only csv files, newest first");
    }

    #[test]
    fn _prune_backups() {
        // by count
        let dir = TempDir::new("prune_count");
        let database = db_with(&dir, list_action(), 2, 0);
        let oldest = make_backup(&database, "a.csv", "", 400);
        let older = make_backup(&database, "b.csv", "", 300);
        let newer = make_backup(&database, "c.csv", "", 200);
        let newest = make_backup(&database, "d.csv", "", 100);

        assert!(database.prune_backups().is_ok());
        assert!(!oldest.exists() && !older.exists(), "Excess backups not removed");
        assert!(newer.exists() && newest.exists(), "Newest backups removed");

        // by age
        let dir = TempDir::new("prune_age");
        let database = db_with(&dir, list_action(), 0, 1);
        let stale = make_backup(&database, "stale.csv", "", 3 * 24 * 60 * 60);
        let fresh = make_backup(&database, "fresh.csv", "", 60);

        assert!(database.prune_backups().is_ok());
        assert!(!stale.exists(), "Old backup not removed");
        assert!(fresh.exists(), "Fresh backup removed");

        // zero limits disable pruning
        let dir = TempDir::new("prune_off");
        let database = db_with(&dir, list_action(), 0, 0);
        let kept = make_backup(&database, "kept.csv", "", 10 * 24 * 60 * 60);

        assert!(database.prune_backups().is_ok());
        assert!(kept.exists(), "Backup removed with limits disabled");
    }

    #[test]
    fn _print_backups() {
        let dir = TempDir::new("print_backups");
        let database = db(&dir);

        assert!(database.print_backups().is_err(), "Missing backups dir should error");

        make_backup(&database, "a.csv", "", 10);
        make_backup(&database, "b.csv", "", 20);

        assert!(database.print_backups().is_ok());
    }
}