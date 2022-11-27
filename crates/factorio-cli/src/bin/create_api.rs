use std::{fs, path::PathBuf};

use clap::Parser;
use factorio_exporter::{load_api, ApiGenerator, Result};
use rust_format::{Config, Edition, Formatter, PostProcess, RustFmt};

/// Generates Rust struct definitions that can be deserialized from Factorio
/// rulesets.
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Directory where Factorio is installed. This needs to be the full
    /// version. Neither the demo nor the headless version are sufficient
    #[arg(long)]
    factorio_dir: PathBuf,

    /// Output path for the generated struct definitions. Will use STDOUT if
    /// omitted
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Whether to format the generated code with `rustfmt`
    #[arg(long, short)]
    format: bool,
}

pub fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let api = load_api(&args.factorio_dir)?;
    let mut code = ApiGenerator::new(&api).generate_api();

    if args.format {
        code = RustFmt::from_config(
            Config::new_str()
                .edition(Edition::Rust2021)
                .post_proc(PostProcess::ReplaceMarkers)
                .option("normalize_doc_attributes", "true")
                .option("wrap_comments", "true")
                .option("comment_width", "80"),
        )
        .format_str(&code)
        .unwrap();
    }

    if let Some(path) = args.output {
        fs::write(path, code)?;
    } else {
        println!("{}", code);
    }

    Ok(())
}
