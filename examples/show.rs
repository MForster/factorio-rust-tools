use std::path::PathBuf;

use factorio_exporter::{
    errors::{FactorioExporterError::FactorioExecutionError, Result},
    export_prototypes,
};

use clap::Parser;
use indoc::printdoc;
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

    let result = export_prototypes(&args.factorio_dir, "de");

    match result {
        Ok(prototypes) => {
            for prototype in prototypes.item_prototypes.values() {
                println!("{}", prototype.name);
            }
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
