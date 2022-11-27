use std::{env, fs::File, io::BufReader};

use clap::Parser;
use eyre::bail;
use factorio_mod_api::{api::ApiToken, ModPortalClient};
use semver::Version;

use crate::App;

/// Download a mod from the mod portal.
#[derive(Debug, Parser)]
pub struct DownloadModCommand {
    /// Name of the mod
    mod_name: String,

    /// Version of the mod
    mod_version: Version,
}

impl DownloadModCommand {
    pub async fn execute(&self, app: &App) -> eyre::Result<()> {
        let token_file = app.api_token_path();
        if !token_file.exists() {
            bail!("API token not found. Please use `fct login` first.");
        }

        let token: ApiToken = serde_json::from_reader(BufReader::new(File::open(token_file)?))?;

        let client = ModPortalClient::new()?;
        client.download_mod(&self.mod_name, &self.mod_version, &token, env::current_dir()?).await?;

        Ok(())
    }
}
