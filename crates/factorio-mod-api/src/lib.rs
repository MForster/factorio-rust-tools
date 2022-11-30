//! A client for the [Factorio](http://www.factorio.com) [mod portal
//! API](https://wiki.factorio.com/Mod_portal_API).

pub mod api;

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use api::{ApiToken, FullModSpec};
use elsa::FrozenMap;
use futures::StreamExt;
use reqwest::Url;
use semver::Version;
use thiserror::Error;
use tracing::info;

use crate::api::LoginResponse;

/// A simple caching client for the [Factorio mod portal
/// API](https://wiki.factorio.com/Mod_portal_API).
pub struct ModPortalClient {
    client: reqwest::Client,
    specs: FrozenMap<String, Box<FullModSpec>>,
    mod_api_base: Url,
    auth_api_base: Url,
}

impl ModPortalClient {
    /// Creates a new client with default configuration.
    pub fn new() -> Result<ModPortalClient> {
        ModPortalClient::with_client(reqwest::Client::default())
    }

    /// Creates a new client with a pre-configured `reqwest::Client`.
    pub fn with_client(client: reqwest::Client) -> Result<ModPortalClient> {
        ModPortalClient::with_base_urls(
            client,
            Url::parse("https://mods.factorio.com").unwrap(),
            Url::parse("https://auth.factorio.com").unwrap(),
        )
    }

    /// Creates a new client, allowing substitution of custom base URLs.
    fn with_base_urls(
        client: reqwest::Client,
        mod_api_base: Url,
        auth_api_base: Url,
    ) -> Result<ModPortalClient> {
        Ok(ModPortalClient { client, specs: FrozenMap::new(), mod_api_base, auth_api_base })
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
            let url = self
                .mod_api_base
                .join(&format!("api/mods/{name}/full"))
                .map_err(|_| FactorioModApiError::InvalidModName { name: name.into() })?;
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
    /// use std::env;
    ///
    /// let client = ModPortalClient::new()?;
    /// let token = client.login("my_user", "my_password").await?;
    /// client.download_mod("my_mod", &Version::parse("1.0.0")?, &token, &env::current_dir()?);
    /// # Ok(())
    /// # }
    //
    /// ```
    pub async fn login(&self, user_name: &str, password: &str) -> Result<ApiToken> {
        info!("logging in with user name '{user_name}'");
        let url = self.auth_api_base.join("api-login").unwrap();
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
    /// use std::env;
    ///
    /// let client = ModPortalClient::new()?;
    /// let token = client.login("my_user", "my_password").await?;
    /// let bytes = client.download_mod("my_mod", &Version::parse("1.0.0")?, &token, &env::current_dir()?).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_mod(
        &self,
        mod_name: &str,
        version: &Version,
        api_token: &ApiToken,
        directory: &Path,
    ) -> Result<PathBuf> {
        info!("downloading version {version} of '{mod_name}' mod");

        let releases = &self.get_mod_spec(mod_name).await?.short_spec.releases;
        let Some(release) = releases.iter().find(|r| r.version == *version) else {
            return Err(FactorioModApiError::InvalidModVersion { version: version.clone() })
        };

        let url = self
            .mod_api_base
            .join(&release.download_url)
            .expect("the mod api shouldn't return invalid URLs");
        let query = [("username", &api_token.username), ("token", &api_token.token)];

        let response = self.client.get(url).query(&query).send().await?;

        let filepath = directory.join(&release.file_name);
        let mut file = File::create(&filepath)?;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
        }

        Ok(filepath)
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

    // Error that is raised if a mod name can't be used to call the API.
    #[error("Invalid mod name: '{name}'")]
    InvalidModName { name: String },

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

    #[error("Error while doing an IO operation")]
    IOError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use chrono::{DateTime, Utc};
    use httpmock::prelude::*;
    use ordered_float::NotNan;
    use pretty_assertions::assert_eq;
    use reqwest::Url;
    use semver::Version;

    use crate::{
        api::{ModManifest, ModMetadata, ModRelease, ModSpec, ModTag},
        ModPortalClient,
    };

    fn setup() -> Result<(MockServer, ModPortalClient), Box<dyn Error>> {
        let server = MockServer::start();
        let client = ModPortalClient::with_base_urls(
            reqwest::Client::default(),
            Url::parse(&server.base_url())?,
            Url::parse(&server.base_url())?,
        )?;

        Ok((server, client))
    }

    fn mock_full(server: &MockServer) -> httpmock::Mock {
        server.mock(|when, then| {
            when.method(GET).path("/api/mods/mymod/full");
            then.status(200)
                .header("content-type", "application/json")
                .body(include_bytes!("tests/full.json"));
        })
    }

    #[tokio::test]
    async fn full_request() -> Result<(), Box<dyn Error>> {
        let (server, client) = setup()?;
        let mock = mock_full(&server);

        let spec = client.get_mod_spec("mymod").await?;

        assert_eq!(
            spec.short_spec,
            ModSpec {
                metadata: ModMetadata {
                    name: "mymod".into(),
                    owner: "someone".into(),
                    summary: "SUMMARY".into(),
                    title: "TITLE".into(),
                    category: Some("general".into()),
                    downloads_count: 42,
                },
                releases: vec![
                    ModRelease {
                        download_url: "/download/mymod/bde93f095d1b53ed019fca5e".into(),
                        file_name: "mymod_0.0.1.zip".into(),
                        info_json: ModManifest {
                            factorio_version: "0.14".into(),
                            dependencies: Some(vec!["base >= 0.13.0".try_into()?]),
                        },
                        released_at: DateTime::from_utc(
                            DateTime::parse_from_rfc3339("2022-06-14T11:45:45.165000Z")?
                                .naive_utc(),
                            Utc
                        ),
                        version: Version::parse("0.0.1")?,
                        sha1: "65b0435dbd4fb0ab0ceea61549641bf6f7dce9d2".into(),
                    },
                    ModRelease {
                        download_url: "/download/mymod/7303634ba9642c5a321a757e".into(),
                        file_name: "mymod_0.0.2.zip".into(),
                        info_json: ModManifest {
                            factorio_version: "0.14".into(),
                            dependencies: Some(vec!["base >= 0.13.0".try_into()?]),
                        },
                        released_at: DateTime::from_utc(
                            DateTime::parse_from_rfc3339("2022-09-24T08:53:02.970000Z")?
                                .naive_utc(),
                            Utc
                        ),
                        version: Version::parse("0.0.2")?,
                        sha1: "a3499805018d6acaa29c1fbeaae763e6d8ae4279".into(),
                    }
                ],
                description: Some("DESCRIPTION".into()),
                github_path: Some("not/existing".into()),
                tag: Some(ModTag { name: "general".into() }),
                score: NotNan::new(88.15f64)?,
                thumbnail: Some(
                    "/assets/6eadeac2dade6347e87c0d24fd455feffa7069f0.thumb.png".into()
                ),
            }
        );

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn request_get_cached() -> Result<(), Box<dyn Error>> {
        let (server, client) = setup()?;
        let mock = mock_full(&server);

        client.get_mod_spec("mymod").await?;
        client.get_mod_spec("mymod").await?;

        mock.assert_hits(1);
        Ok(())
    }
}
