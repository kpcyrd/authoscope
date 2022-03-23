use crate::errors::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub runtime: RuntimeConfig,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub rlimit_nofile: Option<u64>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let home = dirs::home_dir()
            .ok_or_else(|| format_err!("home folder not found"))?;

        for name in &["authoscope", "badtouch"] {
            let path = home.join(&format!(".config/{}.toml", name));
            if path.exists() {
                return Config::from_file(path);
            }
        }

        Ok(Config::default())
    }

    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Config::try_from_str(&buf)
    }

    #[inline]
    pub fn try_from_str(buf: &str) -> Result<Config> {
        let config = toml::from_str(&buf)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_empty() {
        let config = Config::try_from_str("").unwrap();
        assert_eq!(config, Config::default());
    }
}
