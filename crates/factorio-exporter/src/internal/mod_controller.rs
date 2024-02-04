use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use derive_builder::Builder;
use serde_derive::Serialize;
use tracing::debug;

use crate::{FactorioExporterError, Result};

pub struct ModController {
    mods_dir: PathBuf,
}

impl ModController {
    pub fn new(mods_dir: PathBuf) -> ModController {
        ModController { mods_dir }
    }

    #[cfg(unix)]
    fn copy_or_link<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        let from = std::fs::canonicalize(from)?;
        debug!("create symbolic link: {} -> {} ", from.display(), to.as_ref().display());

        std::os::unix::fs::symlink(from, to)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn copy_or_link<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        fs::copy(from, to)?;
        Ok(())
    }

    pub fn add_mod(&self, path: &Path) -> Result<()> {
        if !path.exists() || !path.is_file() {
            return Err(FactorioExporterError::FileNotFoundError { file: path.into() });
        }

        fs::create_dir_all(&self.mods_dir)?;
        Self::copy_or_link(path, self.mods_dir.join(path.file_name().unwrap()))?;
        Ok(())
    }
}

/// The contents of an `info.json` file in a mod. Described [on the
/// Wiki](https://wiki.factorio.com/Tutorial:Mod_structure#info.json).
#[derive(Builder, Serialize)]
#[builder(setter(into, strip_option))]

pub struct ModManifest {
    pub name: String,
    pub version: String,
    pub title: String,
    pub author: String,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[builder(default = r#""1.1".into()"#)]
    pub factorio_version: String,

    #[builder(default)]
    pub dependencies: Vec<String>,
}
