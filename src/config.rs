use crate::error::Error;
use crate::result::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use toml;

const CONFIG_FILE: &'static str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub url: Option<String>,
    pub api_key: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Ok(mut file) = File::open(config_dir()?.join(CONFIG_FILE)) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            toml::from_str(contents.as_str()).map_err(|e| e.into())
        } else {
            Ok(Self {
                url: None,
                api_key: None,
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = config_dir()?;
        create_dir_all(&config_dir)?;
        let mut file = File::create(config_dir.join(CONFIG_FILE))?;
        write!(file, "{}", toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("org", "readmine", "readmine").ok_or(Error::ProjectDirs)
}

fn config_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}
