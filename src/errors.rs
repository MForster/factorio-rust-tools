use thiserror::Error;

#[derive(Error, Debug)]
pub enum FactorioExporterError {
    #[error("API definition file (runtime-api.json) not found")]
    ApiDefinitionNotFound(),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse JSON")]
    JsonParsinError(#[from] serde_json::Error),
}
