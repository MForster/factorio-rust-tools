use std::collections::HashMap;

use eyre::{bail, eyre, Result};
use factorio_mod_api::api::{ModDependency, ModRelease};
use factorio_mod_api::ModPortalClient;
use semver::Version;
use tracing::{debug, trace};

/// A component that tries to find a set of compatible mod versions for a set of
/// given mods and their transitive dependencies. The algorithm uses a greedy
/// heuristic that cannot exhaustively find any potential solution.
///
/// Note: This is pretty hacky at the moment and mostly intended to be a
/// demonstration of the [`ModPortalClient`].
pub struct ModVersionResolver {
    client: ModPortalClient,
    outstanding: Vec<ModDependency>,
    resolutions: HashMap<String, ModRelease>,
}

impl ModVersionResolver {
    pub fn new() -> Result<ModVersionResolver> {
        Ok(ModVersionResolver {
            resolutions: HashMap::new(),
            outstanding: Vec::new(),
            client: ModPortalClient::new()?,
        })
    }

    /// Resolves the dependencies of a set of mods.
    pub async fn resolve(mut self, deps: Vec<ModDependency>) -> Result<HashMap<String, Version>> {
        self.outstanding = deps;
        let mut done = Vec::new();
        while let Some(dep) = self.outstanding.pop() {
            self.resolve_mod(&dep).await?;
            done.push(dep);
        }

        for (mod_name, release) in &self.resolutions {
            for dep in release
                .info_json
                .dependencies
                .as_ref()
                .expect("full ModSpecs must contain dependencies")
                .iter()
                .filter(|d| d.name != "base")
            {
                use factorio_mod_api::api::ModDependencyFlavor::*;
                let dep_name = &dep.name;

                if let Some(resolution) = self.resolutions.get(dep_name) {
                    let version = &resolution.version;

                    if let Incompatibility = dep.flavor {
                        bail!("mod '{mod_name}' is incompatible with '{dep_name}'")
                    }

                    if let Some(c) = &dep.comparator {
                        if !c.matches(version) {
                            bail!("'{mod_name}': dependency '{dep}' is incompatible with '{dep_name}' version {version}' ")
                        }
                    }
                } else if let Normal | NoEffectOnLoadOrder = dep.flavor {
                    bail!("'{mod_name}': required dependency '{dep_name}' missing",)
                }
            }
        }

        Ok(self.resolutions.into_iter().map(|(n, r)| (n, r.version)).collect())
    }

    pub async fn resolve_mod(&mut self, d: &ModDependency) -> Result<()> {
        let name = &d.name;
        debug!("resolving {name}");

        let spec = self.client.get_mod_spec(name).await?;

        let release = spec
            .short_spec
            .releases
            .iter()
            .filter(|r| d.comparator.as_ref().map(|v| v.matches(&r.version)).unwrap_or(true))
            .max_by_key(|r| &r.version)
            .ok_or_else(|| eyre!("{name}: could not resolve version {:?}", d.comparator))?
            .clone();

        for dependency in release
            .info_json
            .dependencies
            .as_ref()
            .expect("full ModSpecs must contain dependencies")
        {
            if dependency.is_required()
                && dependency.name != "base"
                && !self.resolutions.contains_key(&dependency.name)
            {
                let dep_name = &dependency.name;
                trace!("enqueuing {dep_name}");
                self.outstanding.push(dependency.clone());
            }
        }

        self.resolutions.insert(name.into(), release);

        Ok(())
    }
}
