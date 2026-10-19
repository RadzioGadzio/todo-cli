use std::error::Error;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{ErrorKind};
use serde::{ Serialize, Deserialize };
use crate::args::Args;

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub db_path: PathBuf,
    pub backups_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            db_path: Path::new("./.todos.csv").to_path_buf(),
            backups_dir: Path::new("./backups").to_path_buf(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        match fs::read_to_string(&path) {
            Ok(content) if content.trim().is_empty() => Ok(Config::default()),
            Ok(content) => Ok(serde_yaml_ng::from_str(&content)?),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                let config = Config::default();
                config.save(path)?;
                Ok(config)
            },
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_yaml_ng::to_string(self)?)?;
        Ok(())
    }

    pub fn from_args(args: &Args) -> Result<Self, Box<dyn Error>> {
        Self::load(&args.config)
    }
}