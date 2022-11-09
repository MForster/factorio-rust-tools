mod resolver;

use std::{fs, path::PathBuf};

use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand, ValueEnum};
use config::Config;
use directories::ProjectDirs;
use eyre::Result;
use factorio_exporter::{load_api, FactorioExporter, FactorioExporterError};
use factorio_mod_api::api::ModDependency;
use indoc::printdoc;
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info};

use crate::resolver::ModVersionResolver;

/// A collection of tools for Factorio (http://www.factorio.com)
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Directory where Factorio is installed. This needs to be the full
    /// version. Neither the demo nor the headless version are sufficient. This
    /// argument is optional if both of `--factorio-api-spec` and
    /// `--factorio-binary` are specified.
    #[arg(long)]
    factorio_dir: Option<PathBuf>,

    /// Location of the `runtime-api.json` file. Defaults to
    /// `<FACTORIO_DIR>/doc-html/runtime-api.json`.
    ///
    /// The spec can be found in the `doc-html` directory of a full Factorio
    /// installation, or
    /// [online](https://lua-api.factorio.com/latest/runtime-api.json).
    #[arg(long)]
    factorio_api_spec: Option<PathBuf>,

    /// Location of the factorio binary. Defaults to
    /// `<FACTORIO_DIR>/bin/x64/factorio(.exe)`. This can be any Factorio binary
    /// (full, headless, demo).
    #[arg(long)]
    factorio_binary: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Exports prototypes from Factorio in JSON or YAML format
    Export {
        /// Path where the result should be written. Uses STDOUT if not specified.
        #[arg(long, short)]
        destination: Option<PathBuf>,

        /// Format of the output
        #[arg(long, short, default_value = "json")]
        format: OutputFormat,

        /// Export icon paths
        #[arg(long, short)]
        icons: bool,

        /// Mods to install before exporting the prototypes
        mods: Vec<PathBuf>,
    },
    /// Lists all dependencies of a set of mods, trying to find compatible
    /// versions
    ResolveMods {
        /// A list of mods, optionally with version requirements
        mods: Vec<String>,
    },
}

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum OutputFormat {
    Json,
    Yaml,
}

/// The JSON spec of the Factorio mod API, relative to the root directory of a
/// full Factorio installation.
const RUNTIME_API_DEFINITION: &str = "doc-html/runtime-api.json";

/// The path of the Factorio binary, relative to the root directory of a
/// Factorio installation (either full, headless or demo).
#[cfg(windows)]
const FACTORIO_BINPATH: &str = "bin/x64/factorio.exe";
#[cfg(not(windows))]
const FACTORIO_BINPATH: &str = "bin/x64/factorio";

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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let args = Args::parse();
    let mut cmd = Args::command();

    debug!("Parsed arguments: {:?}", args);

    let dirs = ProjectDirs::from("", "", "fct").unwrap();

    let settings: Settings = Config::builder()
        .add_source(config::File::from(dirs.config_dir().join("config")).required(false))
        .add_source(config::Environment::with_prefix("FCT"))
        .build()?
        .try_deserialize()?;

    debug!("Config file contents: {:?}", settings);

    let factorio_dir = args.factorio_dir.or(settings.paths.factorio_dir);

    let api_spec = args
        .factorio_api_spec
        .or(settings.paths.factorio_api_spec)
        .or_else(|| factorio_dir.as_ref().map(|d| d.join(RUNTIME_API_DEFINITION)))
        .map(|path| {
            if !path.exists() {
                cmd.error(
                    ErrorKind::ValueValidation,
                    format!("File not found: '{}'", path.display()),
                )
                .exit()
            };
            path
        })
        .map(std::fs::canonicalize)
        .unwrap_or_else(|| {
            cmd.error(
                ErrorKind::MissingRequiredArgument,
                "One of --factorio-api-spec or --factorio-dir must be specified",
            )
            .exit()
        })?;

    let binary = args
        .factorio_binary
        .or(settings.paths.factorio_binary)
        .or_else(|| factorio_dir.as_ref().map(|d| d.join(FACTORIO_BINPATH)))
        .map(|path| {
            if !path.exists() {
                cmd.error(
                    ErrorKind::ValueValidation,
                    format!("File not found: '{}'", path.display()),
                )
                .exit()
            };
            path
        })
        .map(std::fs::canonicalize)
        .unwrap_or_else(|| {
            cmd.error(
                ErrorKind::MissingRequiredArgument,
                "One of --factorio-binary or --factorio-dir must be specified",
            )
            .exit()
        })?;

    match args.command {
        Commands::Export { destination, format, icons, mods } => {
            let api = load_api(&api_spec)?;
            let exporter = FactorioExporter::new(&binary, &api, "en", icons)?;

            exporter.install_mods(&mods)?;

            match exporter.export() {
                Ok(prototypes) => {
                    let parsed: Value = serde_yaml::from_value(prototypes)?;

                    let output = match format {
                        OutputFormat::Json => serde_json::to_string_pretty(&parsed)?,
                        OutputFormat::Yaml => serde_yaml::to_string(&parsed)?,
                    };

                    info!("write output");
                    match destination {
                        Some(path) => fs::write(path, output)?,
                        None => println!("{}", output),
                    }
                }

                Err(FactorioExporterError::FactorioExecutionError { stdout, stderr }) => {
                    printdoc! {r"
                        Failed to execute Factorio:
                        === STDOUT
                        {}
                        === STDERR
                        {}
                    ", stdout, stderr};
                }
                Err(e) => return Err(e.into()),
            }
        }
        Commands::ResolveMods { mods } => {
            let mods: factorio_mod_api::Result<Vec<ModDependency>> =
                mods.iter().map(|a| ModDependency::try_from(a.as_str())).collect();

            let resolutions = ModVersionResolver::new().unwrap().resolve(mods?).await?;

            for (mod_name, version) in resolutions.iter().sorted_by_key(|&(name, _)| name) {
                println!("{mod_name} {version}");
            }
        }
    }

    info!("done");

    Ok(())
}
