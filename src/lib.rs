mod api;
mod driver;
pub mod errors;
mod exporter_script_builder;
mod mod_controller;
mod prototypes;

use std::{fs::File, io::BufReader, path::Path};

use errors::FactorioExporterError;
use prototypes::PrototypeExport;
use tracing::debug;

use crate::{api::Api, driver::export};

const RUNTIME_API_DEFINITION: &str = "doc-html/runtime-api.json";

pub fn export_prototypes(
    factorio_dir: &Path,
    locale: &str,
) -> Result<PrototypeExport, FactorioExporterError> {
    let api_file_path = factorio_dir.join(RUNTIME_API_DEFINITION);

    debug!(
        "Loading API definition file from {}",
        &api_file_path.display()
    );

    let api_def: Api = serde_json::from_reader(BufReader::new(File::open(api_file_path)?))?;

    debug!(
        "parsed API, got {} classes and {} concepts",
        &api_def.classes.len(),
        &api_def.concepts.len()
    );

    export(factorio_dir, locale)
}
