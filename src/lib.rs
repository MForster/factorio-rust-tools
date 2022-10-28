pub mod api;
pub mod errors;

use std::{fs::File, io::BufReader, path::Path};

use errors::FactorioExporterError;
use tracing::debug;

use crate::api::Api;

const RUNTIME_API_DEFINITION: &str = "doc-html/runtime-api.json";

pub fn export_prototypes(factorio_dir: &Path) -> Result<String, FactorioExporterError> {
    let api_file_path = factorio_dir.join(RUNTIME_API_DEFINITION);

    debug!(
        "Loading API definition file from {}",
        &api_file_path.display()
    );

    let api_def: Api = serde_json::from_reader(BufReader::new(File::open(api_file_path)?))?;

    Ok(format!(
        "Imported {} classes and {} concepts",
        &api_def.classes.len(),
        &api_def.concepts.len()
    ))
}
