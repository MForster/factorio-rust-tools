use std::path::{Path, PathBuf};

use config::Config;
use eyre::Result;
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub paths: PathSettings,
}

#[derive(Debug, Default, Deserialize)]
pub struct PathSettings {
    pub factorio_dir: Option<PathBuf>,
    pub factorio_api_spec: Option<PathBuf>,
    pub factorio_binary: Option<PathBuf>,
}

impl Settings {
    pub fn init(path: &Path) -> Result<Settings> {
        let settings: Settings = Config::builder()
            .add_source(config::File::from(path).required(false))
            .build()?
            .try_deserialize()?;

        debug!("Config file contents: {:?}", settings);

        Ok(settings)
    }
}
