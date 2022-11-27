//! A library and binary for exporting object prototype definitions from
//! [Factorio](https://www.factorio.com) in JSON or YAML.
//!
//! Usage:
//! ```no_run
//! use factorio_exporter::{ load_api, FactorioExporter, FactorioExporterError, Result };
//! use std::path::PathBuf;
//!
//! let api_spec = PathBuf::from("/home/user/factorio/doc-html/runtime-api.json");
//! let factorio_binary = PathBuf::from("/home/user/factorio/bin/x64/factorio");
//!
//! let api = load_api(&api_spec)?;
//! let exporter = FactorioExporter::new(&factorio_binary, &api, "en", true)?;
//!
//! let result: serde_yaml::Value = exporter.export()?;
//!
//! # Ok::<(), FactorioExporterError>(())
//! ```
//!
//! The result is returned as a [`serde_yaml::Value`] object, which can easily
//! deserialized of serialized into other data types further. See this [example]
//! to see the structure that the data has.
//!
//! [example]:
//!     https://raw.githubusercontent.com/MForster/factorio-rust-tools/main/crates/factorio-exporter/data/vanilla.json
#![deny(unused_must_use)]
use std::path::PathBuf;

use thiserror::Error;

pub use api::load_api;
pub use exporter::FactorioExporter;

mod api;
mod api_generator;
mod exporter;
mod internal;

/// Main result type used throughout factorio-explorer
pub type Result<T> = std::result::Result<T, FactorioExporterError>;

/// Main error type used throughout factorio-explorer
#[derive(Error, Debug)]
pub enum FactorioExporterError {
    /// Error that is raised if Factorio could not be started to execute the
    /// exporter mods. The process output to stdout and stderr is saved in the
    /// error object.
    #[error("error while executing Factorio")]
    FactorioExecutionError { stdout: String, stderr: String },

    /// Error that is raised if Factorio's output could not be parsed. This can
    /// have all kinds of root causes, but the underlying reason should normally
    /// be apparent from the process output stored in this error object.
    #[error("failed to parse Factorio output")]
    FactorioOutputError { message: String, output: String },

    /// Error that is raised if a file couldn't be found, for example the API
    /// spec or the Factorio binary. This is usually a user error.
    #[error("{file} does not exist or isn't a file")]
    FileNotFoundError { file: PathBuf },

    /// Error that is raised if the user specified conflicting or incomplete
    /// command line arguments.
    #[error("{0}")]
    InvocationError(String),

    /// Error that is raised if a file system operation failed unexpectedly.
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    /// Error that is raised if deserialization from JSON failed.
    #[error("failed to parse JSON")]
    JsonParsingError(#[from] serde_json::Error),

    /// Error that is raised if deserialization from JSON failed.
    #[error("failed to parse YAML")]
    YamlParsingError(#[from] serde_yaml::Error),
}
