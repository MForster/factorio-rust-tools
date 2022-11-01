use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use factorio_exporter::{load_api, FactorioExporter, FactorioExporterError, Result};
use indoc::printdoc;
use serde_json::Value;
use tracing::{debug, info};

/// Exports prototypes from Factorio in JSON or YAML format
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Directory where Factorio is installed. This needs to be the full
    /// version. Neither the demo nor the headless version are sufficient
    factorio_dir: PathBuf,

    /// Path where the result should be written. Uses STDOUT if not specified
    #[arg(long, short)]
    destination: Option<PathBuf>,

    /// Format of the output
    #[arg(long, short, default_value = "json")]
    format: OutputFormat,
}

#[derive(Clone, Debug, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum OutputFormat {
    Json,
    Yaml,
}

fn main() -> Result<()> {
    let args = Args::parse();
    debug!("Parsed arguments: {:?}", args);

    let api = load_api(&args.factorio_dir)?;
    let exporter = FactorioExporter::new(&args.factorio_dir, &api, "en")?;

    match exporter.export() {
        Ok(prototypes) => {
            let parsed: Value = serde_yaml::from_str(&prototypes)?;

            let output = match args.format {
                OutputFormat::Json => serde_json::to_string_pretty(&parsed)?,
                OutputFormat::Yaml => serde_yaml::to_string(&parsed)?,
            };

            info!("write output");
            match args.destination {
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
        Err(e) => return Err(e),
    }

    info!("done");

    Ok(())
}
