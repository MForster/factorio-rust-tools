pub mod errors;

use std::{fs::File, io::BufReader, path::Path};

use errors::FactorioExporterError;
use itertools::Itertools;
use serde_json::Value;
use tracing::debug;

const RUNTIME_API_DEFINITION: &str = "doc-html/runtime-api.json";

pub fn export_prototypes(factorio_dir: &Path) -> Result<String, FactorioExporterError> {
    let api_file_path = factorio_dir.join(RUNTIME_API_DEFINITION);

    debug!(
        "Loading API definition file from {}",
        &api_file_path.display()
    );

    let api_def: Value = serde_json::from_reader(BufReader::new(File::open(api_file_path)?))?;
    let keys = api_def.as_object().unwrap().keys().collect_vec();

    Ok(format!(
        "Found {} top-level keys in definition file: {:?}",
        keys.len(),
        &keys
    ))
}
