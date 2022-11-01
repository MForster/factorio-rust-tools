use std::path::PathBuf;

use clap::Parser;
use factorio_exporter::{export, load_api, FactorioExporterError::FactorioExecutionError, Result};
use indoc::printdoc;
use itertools::Itertools;
use tracing::debug;

/// Example that shows how to call Factorio Exporter.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Directory where Factorio is installed. This needs to be the full
    /// version. Neither the demo nor the headless version are sufficient.
    #[arg(long)]
    factorio_dir: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("Parsed arguments: {:?}", args);

    let api = load_api(&args.factorio_dir)?;
    let result = export(&args.factorio_dir, &api, "en");

    match result {
        Ok(prototypes) => {
            println!(
                "Found {} items, here are a few: {:?}",
                prototypes.item_prototypes.len(),
                prototypes.item_prototypes.values().take(5).format(", ")
            );
        }
        Err(FactorioExecutionError { stdout, stderr }) => {
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

    Ok(())
}
