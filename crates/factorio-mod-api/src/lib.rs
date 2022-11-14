//! A client for the [Factorio](http://www.factorio.com) [mod portal
//! API](https://wiki.factorio.com/Mod_portal_API).

pub mod api;

use api::{ApiToken, FullModSpec};
use bytes::Bytes;
use elsa::FrozenMap;
use semver::Version;
use thiserror::Error;
use tracing::info;

use crate::api::LoginResponse;

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
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use factorio_mod_api::ModPortalClient;
    ///
    /// let client = ModPortalClient::new()?;
    /// let spec = client.get_mod_spec("my_mod").await?;
    /// println!("{}", spec.created_at);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_mod_spec(&self, name: &str) -> Result<&FullModSpec> {
        Ok(if let Some(spec) = self.specs.get(name) {
            info!("returning mod spec for '{name}' from cache");
            spec
        } else {
            info!("requesting mod spec for '{name}'");
            let url = format!("https://mods.factorio.com/api/mods/{name}/full");
            let response = self.client.get(url).send().await?.json().await?;
            self.specs.insert(name.into(), Box::new(response))
        })
    }

    /// Get a login token needed to invoke authenticated APIs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use factorio_mod_api::ModPortalClient;
    /// use semver::Version;
    ///
    /// let client = ModPortalClient::new()?;
    /// let token = client.login("my_user", "my_password").await?;
    /// client.download_mod("my_mod", &Version::parse("1.0.0")?, &token);
    /// # Ok(())
    /// # }
    //
    /// ```
    pub async fn login(&self, user_name: &str, password: &str) -> Result<ApiToken> {
        info!("logging in with user name '{user_name}'");

        let url = "https://auth.factorio.com/api-login";
        let query = [("api_version", "4"), ("username", user_name), ("password", password)];

        let request = self.client.post(url).query(&query);
        let response = request.send().await?.json().await?;

        match response {
            LoginResponse::Success { token } => Ok(token),
            LoginResponse::Error { error, message } => {
                Err(FactorioModApiError::LoginError { error, message })
            }
        }
    }

    /// Download a mod from the mod portal.
    ///
    /// This is an authenticated endpoint that needs a login token to be
    /// obtained with [`ModPortalClient::login`] first.
    ///
    /// # Example
    /// ```no_run
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// use factorio_mod_api::ModPortalClient;
    /// use semver::Version;
    ///
    /// let client = ModPortalClient::new()?;
    /// let token = client.login("my_user", "my_password").await?;
    /// let bytes = client.download_mod("my_mod", &Version::parse("1.0.0")?, &token).await?;
    /// std::fs::write("my_mod_1.0.0.zip", bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_mod(
        &self,
        mod_name: &str,
        version: &Version,
        api_token: &ApiToken,
    ) -> Result<Bytes> {
        info!("downloading version {version} of '{mod_name}' mod");

        let releases = &self.get_mod_spec(mod_name).await?.short_spec.releases;
        let Some(release) = releases.iter().find(|r| r.version == *version) else {
            return Err(FactorioModApiError::InvalidModVersion { version: version.clone() })
        };

        let url = format!("https://mods.factorio.com/{}", release.download_url);
        let query = [("username", &api_token.username), ("token", &api_token.token)];

        Ok(self.client.get(url).query(&query).send().await?.bytes().await?)
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

    // Error that is raised if a mod version doesn't exist.
    #[error("Invalid mod version: '{version}'")]
    InvalidModVersion { version: Version },

    /// Error that is raised if a request to the mod portal failed.
    #[error("Error while talking to the API Server")]
    RequestError(#[from] reqwest::Error),

    /// Error that is raised if parsing of a SemVer version number failed.
    #[error("Error while parsing a version number")]
    VersionError(#[from] semver::Error),

    /// Error that is raised if deserialization from JSON failed.
    #[error("failed to parse JSON")]
    JsonParsingError(#[from] serde_json::Error),

    #[error("failed to log in: {error}, {message}")]
    LoginError { error: String, message: String },
}
