use errors::{Result, ResultExt};

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use libc::rlim_t;
use toml;


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
    pub rlimit_nofile: Option<rlim_t>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let home = env::home_dir()
                        .chain_err(|| "home folder not found")?;

        let path = home.join(".config/badtouch.toml");

        if path.exists() {
            Config::from_file(path)
        } else {
            Ok(Config::default())
        }
    }

    #[inline]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(path)?;

        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Config::from_str(&buf)
    }

    #[inline]
    pub fn from_str(buf: &str) -> Result<Config> {
        let config = toml::from_str(&buf)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_empty() {
        let config = Config::from_str("").unwrap();
        assert_eq!(config, Config::default());
    }
}
