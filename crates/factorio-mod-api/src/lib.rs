// TODO: https://wiki.factorio.com/Web_authentication_API
// TODO: Download: https://mods.factorio.com/{download_url}?username={username}&token={token}

pub mod api;

use api::FullModSpec;
use elsa::FrozenMap;
use thiserror::Error;
use tracing::info;

/// A simple caching client for the [Factorio mod portal
/// API](https://wiki.factorio.com/Mod_portal_API).
pub struct ModPortalClient {
    client: reqwest::Client,
    specs: FrozenMap<String, Box<FullModSpec>>,
}

impl ModPortalClient {
    /// Creates a new client with default configuration.
    pub fn new() -> Result<ModPortalClient> {
        ModPortalClient::with_client(reqwest::Client::builder().build()?)
    }

    /// Creates a new client with a pre-configured `reqwest::Client`.
    pub fn with_client(client: reqwest::Client) -> Result<ModPortalClient> {
        Ok(ModPortalClient { client, specs: FrozenMap::new() })
    }

    /// Get the full spec of a Factorio mod. Request results are cached in memory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use factorio_mod_api::ModPortalClient;
    /// use futures::TryFutureExt;
    ///
    /// let client = ModPortalClient::new()?;
    /// client.get_mod_spec("Warehousing")
    ///     .map_ok(|spec| println!("{}", spec.created_at));
    ///
    /// # Ok::<(), factorio_mod_api::FactorioModApiError>(())
    /// ```
    pub async fn get_mod_spec(&self, name: &str) -> Result<&FullModSpec> {
        Ok(if let Some(spec) = self.specs.get(name) {
            spec
        } else {
            info!("requesting mod spec for '{name}'");
            let url = format!("https://mods.factorio.com/api/mods/{name}/full");
            let response = self.client.get(url).send().await?.json().await?;
            self.specs.insert(name.into(), Box::new(response))
        })
    }
}

/// Main result type used throughout factorio-mod-api
pub type Result<T> = std::result::Result<T, FactorioModApiError>;

/// Main error type used throughout factorio-mod-api
#[derive(Error, Debug)]
pub enum FactorioModApiError {
    // Error that is raised if a mod dependency has an invalid format.
    #[error("Invalid mod dependency: '{dep}'")]
    InvalidModDependency { dep: String },

    /// Error that is raised if a request to the mod portal failed.
    #[error("Error while talking to the API Server")]
    RequestError(#[from] reqwest::Error),

    /// Error that is raised if parsing of a SemVer version number failed.
    #[error("Error while parsing a version number")]
    VersionError(#[from] semver::Error),

    /// Error that is raised if deserialization from JSON failed.
    #[error("failed to parse JSON")]
    JsonParsingError(#[from] serde_json::Error),
}
