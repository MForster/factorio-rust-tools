use thiserror::Error;

pub use api::load_api;
pub use driver::export;

mod api;
mod driver;
mod internal;
mod prototypes;

#[derive(Error, Debug)]
pub enum FactorioExporterError {
    #[error("API definition file (runtime-api.json) not found")]
    ApiDefinitionNotFound(),
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse JSON")]
    JsonParsingError(#[from] serde_json::Error),
    #[error("failed to parse YAML")]
    YamlParsingError(#[from] serde_yaml::Error),
    #[error("error while executing Factorio")]
    FactorioExecutionError { stdout: String, stderr: String },
}

pub type Result<T> = std::result::Result<T, FactorioExporterError>;
