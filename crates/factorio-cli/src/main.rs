mod commands;
mod settings;

use std::path::PathBuf;

use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use commands::{
    download_mod::DownloadModCommand, export::ExportCommand, login::LoginCommand,
    resolve_mods::ResolveModsCommand,
};
use directories::ProjectDirs;
use eyre::Result;
use settings::Settings;
use tracing::info;

/// A collection of tools for Factorio (<http://www.factorio.com>)
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Directory where Factorio is installed. This needs to be the full
    /// version. Neither the demo nor the headless version are sufficient. This
    /// argument is optional if `--factorio-binary` is specified.
    #[arg(long)]
    factorio_dir: Option<PathBuf>,

    /// Location of the factorio binary. Defaults to
    /// `<FACTORIO_DIR>/bin/x64/factorio(.exe)`. This can be any Factorio binary
    /// (full, headless, demo).
    #[arg(long)]
    factorio_binary: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Export(ExportCommand),
    ResolveMods(ResolveModsCommand),
    DownloadMod(DownloadModCommand),
    Login(LoginCommand),
}

pub struct App {
    dirs: ProjectDirs,
    settings: Settings,
    args: Args,
}

/// The path of the Factorio binary, relative to the root directory of a
/// Factorio installation (either full, headless or demo).
#[cfg(windows)]
const FACTORIO_BINPATH: &str = "bin/x64/factorio.exe";
#[cfg(not(windows))]
const FACTORIO_BINPATH: &str = "bin/x64/factorio";

const TOOL_NAME: &str = "fct";
const CONFIG_NAME: &str = "config";

impl App {
    fn new() -> Result<App> {
        let dirs = ProjectDirs::from("", "", TOOL_NAME).unwrap();
        let settings = Settings::init(&dirs.config_dir().join(CONFIG_NAME))?;
        Ok(App { dirs, settings, args: Args::parse() })
    }

    async fn run(self) -> Result<()> {
        match &self.args.command {
            Commands::Export(cmd) => cmd.execute(&self).await?,
            Commands::ResolveMods(cmd) => cmd.execute(&self).await?,
            Commands::DownloadMod(cmd) => cmd.execute(&self).await?,
            Commands::Login(cmd) => cmd.execute(&self).await?,
        }
        Ok(())
    }

    fn factorio_dir(&self) -> Option<&PathBuf> {
        self.args.factorio_dir.as_ref().or(self.settings.paths.factorio_dir.as_ref())
    }

    fn factorio_binary(&self) -> Result<PathBuf> {
        Ok(self
            .args
            .factorio_binary
            .clone()
            .or_else(|| self.settings.paths.factorio_binary.clone())
            .or_else(|| self.factorio_dir().map(|d| d.join(FACTORIO_BINPATH)))
            .inspect(|path| {
                if !path.exists() {
                    Args::command()
                        .error(
                            ErrorKind::ValueValidation,
                            format!("File not found: '{}'", path.display()),
                        )
                        .exit()
                };
            })
            .map(std::fs::canonicalize)
            .unwrap_or_else(|| {
                Args::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "One of --factorio-binary or --factorio-dir must be specified",
                    )
                    .exit()
            })?)
    }

    fn api_token_path(&self) -> PathBuf {
        self.dirs.config_dir().join("api_token.json")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    App::new()?.run().await?;

    info!("done");

    Ok(())
}
