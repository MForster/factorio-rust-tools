use std::{error::Error, path::PathBuf, process::exit};

use factorio_exporter::export_prototypes;

use clap::Parser;
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

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    debug!("Parsed arguments: {:?}", args);

    println!("{:?}", export_prototypes(&args.factorio_dir)?);

    Ok(())
}
