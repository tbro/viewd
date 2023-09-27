use anyhow::Result;
use config::Config as Configurator;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::DEFAULT_PORT;
#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(rename = "tls-key-file")]
    pub key: PathBuf,
    #[serde(rename = "tls-cert-file")]
    pub cert: PathBuf,
    #[serde(rename = "ca-file")]
    pub ca: PathBuf,
    pub host: String,
}

impl Config {
    pub fn new(path: &Path) -> Result<Arc<Self>> {
        let settings = Configurator::builder()
            .add_source(config::File::from(path))
            .add_source(config::Environment::with_prefix("VIEWD"))
            .build()?;

        let config = settings.try_deserialize::<Config>()?;
        Ok(Arc::new(config))
    }
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_port() -> Result<()> {
        let c = Config::new(&Path::new("config/client/example.toml"))?;
        assert_eq!(DEFAULT_PORT, c.port);
        Ok(())
    }
}
