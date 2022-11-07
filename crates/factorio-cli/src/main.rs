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

fn main() -> Result<()> {
    let args = Args::parse();
    debug!("Parsed arguments: {:?}", args);

    let api_spec = std::fs::canonicalize(
        args.factorio_api_spec
            .or_else(|| args.factorio_dir.as_ref().map(|d| d.join(RUNTIME_API_DEFINITION)))
            .ok_or_else(|| {
                FactorioExporterError::InvocationError(
                    "One of --factorio-api-spec or --factorio-dir must be specified".into(),
                )
            })?,
    )?;

    let binary = std::fs::canonicalize(
        args.factorio_binary
            .or_else(|| args.factorio_dir.as_ref().map(|d| d.join(FACTORIO_BINPATH)))
            .ok_or_else(|| {
                FactorioExporterError::InvocationError(
                    "One of --factorio-binary or --factorio-dir must be specified".into(),
                )
            })?,
    )?;

    let api = load_api(&api_spec)?;
    let exporter = FactorioExporter::new(&binary, &api, "en", args.icons)?;

    exporter.install_mods(&args.mods)?;

    match exporter.export() {
        Ok(prototypes) => {
            let parsed: Value = serde_yaml::from_value(prototypes)?;

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
