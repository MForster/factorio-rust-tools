use std::path::PathBuf;

use factorio_exporter::{load_api, FactorioExporter, FactorioExporterError, Result};

use clap::Parser;
use indoc::printdoc;
use serde_json::Value;
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
    let exporter = FactorioExporter::new(&args.factorio_dir, &api, "en")?;

    match exporter.export() {
        Ok(prototypes) => {
            let parsed: Value = serde_yaml::from_str(&prototypes)?;

            println!(
                "Found {} items, {} recipes, and {} technologies, here are a few:",
                parsed["item_prototypes"].as_object().unwrap().len(),
                parsed["recipe_prototypes"].as_object().unwrap().len(),
                parsed["technology_prototypes"].as_object().unwrap().len(),
            );
            println!("{:#?}", parsed["item_prototypes"]["iron-plate"]);
            println!("{:#?}", parsed["recipe_prototypes"]["iron-plate"]);
            println!("{:#?}", parsed["technology_prototypes"]["logistics-2"]);
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

    Ok(())
}
