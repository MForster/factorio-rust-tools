use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use eyre::Result;
use factorio_exporter::{load_api, FactorioExporter, FactorioExporterError};
use indoc::printdoc;
use serde_yaml::Value;
use tracing::{debug, info};

use crate::App;

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum OutputFormat {
    Json,
    Yaml,
}

/// Exports prototypes from Factorio in JSON or YAML format
#[derive(Debug, Parser)]
pub struct ExportCommand {
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
}

impl ExportCommand {
    pub async fn execute(&self, app: &App) -> Result<()> {
        debug!("Parsed arguments: {:?}", self);

        let api = load_api(&app.api_spec()?)?;
        let binary = app.factorio_binary()?;
        let exporter = FactorioExporter::new(&binary, &api, "en", self.icons)?;

        exporter.install_mods(&self.mods)?;

        match exporter.export() {
            Ok(prototypes) => {
                let parsed: Value = serde_yaml::from_value(prototypes)?;

                let output = match self.format {
                    OutputFormat::Json => serde_json::to_string_pretty(&parsed)?,
                    OutputFormat::Yaml => serde_yaml::to_string(&parsed)?,
                };

                info!("write output");
                match &self.destination {
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

        Ok(())
    }
}
