//! Data types used in the Mod Portal API.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use regex_macro::regex;
use semver::Version;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use strum_macros::{Display, EnumString};
use url::Url;

use crate::{FactorioModApiError, Result};

/// A mod as returned from the `https://mods.factorio.com/api/mods` endpoint
/// (not yet implemented).
#[derive(Debug, Deserialize)]
pub struct ModListing {
    /// Metadata shared between the three different API invocations.
    #[serde(flatten)]
    pub metadata: ModMetadata,

    /// The latest version of the mod available for download.
    pub latest_release: ModRelease,
}

/// A mod as returned from the `https://mods.factorio.com/api/mods/{name}/full`
/// endpoint.
#[derive(Debug, Deserialize)]
pub struct FullModSpec {
    /// Spec data shared with short spec request.
    #[serde(flatten)]
    pub short_spec: ModSpec,

    /// A string describing the recent changes to a mod
    pub changelog: String,

    /// ISO 6501 for when the mod was created.
    pub created_at: DateTime<Utc>,

    /// Usually a URL to the mod's main project page, but can be any string.
    pub homepage: String,

    // Undocumented
    pub images: Vec<ModImage>,
    pub license: ModLicense,
    pub updated_at: DateTime<Utc>,
    pub source_url: Option<Url>,
    pub faq: Option<String>,
}

/// Mod metadata shared between the three different API invocations.
#[derive(Debug, Deserialize)]
pub struct ModMetadata {
    /// The mod's machine-readable ID string.
    pub name: String,

    /// The Factorio username of the mod's author.
    pub owner: String,

    /// A shorter mod description.
    pub summary: String,

    /// The mod's human-readable name.
    pub title: String,

    /// A single tag describing the mod. Warning: Seems to be absent sometimes.
    pub category: Option<String>,

    /// Number of downloads.
    pub downloads_count: u64,
}
/// A mod as returned from the `https://mods.factorio.com/api/mods/{name}`
/// endpoint (not yet implemented). Also returned as part of the full request.
#[derive(Debug, Deserialize)]
pub struct ModSpec {
    /// Metadata shared between the three different API invocations.
    #[serde(flatten)]
    pub metadata: ModMetadata,

    /// A list of different versions of the mod available for download.
    pub releases: Vec<ModRelease>,

    /// A longer description of the mod, in text only format.
    pub description: Option<String>,

    /// A link to the mod's github project page, just prepend "github.com/". Can
    /// be blank ("").
    pub github_path: Option<String>,

    /// A list of tag objects that categorize the mod.
    pub tag: Option<ModTag>,

    // Undocumented
    pub score: f64,
    pub thumbnail: Option<String>,
}

/// A tag object that categorizes a mod.
#[derive(Debug, Deserialize)]
pub struct ModTag {
    /// An all lower-case string used to identify this tag internally.
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ModImage {
    pub id: String,
    pub thumbnail: String,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
pub struct ModLicense {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub url: Url,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModRelease {
    /// Path to download for a mod. starts with "/download" and does not include
    /// a full url.
    pub download_url: String,

    /// The file name of the release. Always seems to follow the pattern
    /// "{name}_{version}.zip"
    pub file_name: String,

    /// A copy of the mod's info.json file, only contains factorio_version in
    /// short version, also contains an array of dependencies in full version
    pub info_json: ModManifest,

    /// ISO 6501 for when the mod was released.
    pub released_at: DateTime<Utc>,

    /// The version string of this mod release. Used to determine dependencies.
    #[serde(deserialize_with = "parse_version")]
    pub version: Version,

    /// The sha1 key for the file
    pub sha1: String,
}

/// Deserializing visitor for `Version` fields.
struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a semver version string (potentially with leading zeros)")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
        Version::parse(&regex!(r#"\.0+([1-9])"#).replace_all(v, ".$1"))
            .map_err(|e| E::custom(e.to_string()))
    }
}

fn parse_version<'de, D>(d: D) -> std::result::Result<Version, D::Error>
where
    D: Deserializer<'de>,
{
    d.deserialize_str(VersionVisitor)
}

/// Partial contents of the `info.json` file that describes a mod.
/// <https://wiki.factorio.com/Tutorial:Mod_structure#info.json>
#[derive(Clone, Debug, Deserialize)]
pub struct ModManifest {
    pub factorio_version: String, // Doesn't parse as semver::Version (no patch level).

    /// The mod's dependencies. Only available in "full" API calls.
    pub dependencies: Option<Vec<ModDependency>>,
}

/// A dependency specification between mods.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(try_from = "&str")]
pub struct ModDependency {
    pub flavor: ModDependencyFlavor,
    pub name: String,
    pub comparator: Option<semver::Comparator>,
}

impl ModDependency {
    pub fn unversioned(name: String) -> ModDependency {
        ModDependency { flavor: ModDependencyFlavor::Normal, name, comparator: None }
    }
}

#[derive(Clone, Debug, Display, EnumString, Eq, PartialEq)]
pub enum ModDependencyFlavor {
    #[strum(serialize = "")]
    Normal,
    #[strum(serialize = "!")]
    Incompatibility,
    #[strum(serialize = "?")]
    Optional,
    #[strum(serialize = "(?)")]
    Hidden,
    #[strum(serialize = "~")]
    NoEffectOnLoadOrder,
}

impl TryFrom<&str> for ModDependency {
    type Error = FactorioModApiError;

    fn try_from(value: &str) -> Result<Self> {
        let re = regex!(
            r#"(?x)^
            (?: (?P<prefix> ! | \? | \(\?\) | ~ ) \s*)?
            (?P<name> [[[:alnum:]]-_][[[:alnum:]]-_\ ]{1, 48}[[[:alnum:]]-_])
            (?P<comparator>
                \s* (?: < | <= | = | >= | > )
                \s* \d{1,5}\.\d{1,5}(\.\d{1,5})?
            )?
            $"#,
        );

        let caps = re
            .captures(value)
            .ok_or_else(|| FactorioModApiError::InvalidModDependency { dep: value.into() })?;

        Ok(ModDependency {
            flavor: caps
                .name("prefix")
                .map(|prefix| ModDependencyFlavor::from_str(prefix.as_str()).unwrap())
                .unwrap_or(ModDependencyFlavor::Normal),
            name: caps["name"].into(),
            comparator: caps
                .name("comparator")
                .map(|comparator| semver::Comparator::parse(comparator.as_str()))
                .transpose()?,
        })
    }
}

impl Display for ModDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.flavor)?;
        if self.flavor != ModDependencyFlavor::Normal {
            write!(f, " ")?;
        }

        write!(f, "{}", self.name)?;

        if let Some(comparator) = &self.comparator {
            use semver::Op::*;
            let op = match comparator.op {
                Exact => "=",
                Greater => ">",
                GreaterEq => ">=",
                Less => "<",
                LessEq => "<=",
                _ => unimplemented!(),
            };
            write!(
                f,
                " {op} {}.{}.{}",
                comparator.major,
                comparator.minor.unwrap(),
                comparator.patch.unwrap()
            )?;
        }

        Ok(())
    }
}

impl ModDependency {
    pub fn is_required(&self) -> bool {
        use ModDependencyFlavor::*;
        [Normal, NoEffectOnLoadOrder].contains(&self.flavor)
    }
}

/// A token that identifies a logged in user that needs to be passed to API
/// calls that require login.
///
/// Use [`ModPortalClient::login`] to obtain.
#[derive(Debug, Deserialize, Serialize)]
pub struct ApiToken {
    pub token: String,
    pub username: String,
}

/// Response from the `login` endpoint.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LoginResponse {
    Success {
        #[serde(flatten)]
        token: ApiToken,
    },
    Error {
        error: String,
        message: String,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::{ModDependency, ModDependencyFlavor};
    use semver::Comparator;

    #[test]
    fn basic() -> Result<()> {
        let d: ModDependency = serde_json::from_str(r#""? some-mod-everyone-loves >= 4.2.0""#)?;
        assert!(
            d == ModDependency {
                flavor: ModDependencyFlavor::Optional,
                name: "some-mod-everyone-loves".into(),
                comparator: Some(Comparator::parse(">= 4.2.0")?),
            }
        );
        Ok(())
    }
}
