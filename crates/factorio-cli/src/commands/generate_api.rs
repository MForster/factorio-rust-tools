use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use clap::Parser;
use color_eyre::Result;
use factorio_exporter::{load_api, ApiGenerator};
use rust_format::{Config, Edition, Formatter, PostProcess, RustFmt};

use crate::{api::Prototypes, App};

/// Generates Rust struct definitions that can be deserialized from Factorio
/// rule sets.
#[derive(Debug, Parser)]
pub struct GenerateApiCommand {
    /// Output path for the generated struct definitions. Will use STDOUT if
    /// omitted
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Whether to format the generated code with `rustfmt`
    #[arg(long, short)]
    format: bool,
}

impl GenerateApiCommand {
    pub async fn execute(&self, app: &App) -> Result<()> {
        let api = load_api(&app.api_spec()?)?;
        let api_generator = ApiGenerator::new(&api);

        let mut code = api_generator.generate().to_string();

        if self.format {
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

        if let Some(path) = &self.output {
            fs::write(path, code)?;
        } else {
            println!("{}", code);
        }

        let _prototypes: Prototypes = serde_json::from_reader(BufReader::new(File::open(
            "/home/mforster/factorio-rust-tools/crates/factorio-exporter/data/vanilla.json",
        )?))?;

        Ok(())
    }
}
