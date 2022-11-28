use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use convert_case::{Case, Casing};
use indoc::writedoc;

use regex::Captures;
use regex_macro::regex;
use serde_derive::Deserialize;
use serde_yaml::Value;
use tempfile::TempDir;
use tracing::{debug, error, info};

use crate::{
    api::Api,
    internal::{
        mod_controller::{ModController, ModManifestBuilder},
        script_generator::ScriptGenerator,
    },
    FactorioExporterError::{self, FactorioExecutionError},
    Result,
};

const CONFIG: &str = "config.ini";
const SAVE: &str = "save.zip";
const MOD_NAME: &str = "factorio_exporter";
const MOD_VERSION: &str = "0.0.1";
const MODS_DIR: &str = "mods";

/// Main class for orchestrating the export.
pub struct FactorioExporter<'a> {
    factorio_binary: &'a Path,
    api: &'a Api,
    locale: &'a str,
    temp_dir: TempDir,
    mod_controller: ModController,
    export_icons: bool,
}

impl FactorioExporter<'_> {
    /// Creates and configures a new `FactorioExporter` instance.
    ///
    /// # Arguments
    ///
    /// * `factorio_binary` - File system path of a Factorio binary. This can be
    ///   any variant of the binary, full, headless, or demo.
    /// * `api` - The definition of the Factorio API, as loaded by
    ///   [`load_api`](super::load_api).
    /// * `locale` - Locale code to use for translated strings.
    /// * `export_icons` - Whether icon paths should be collected in the data
    ///   phase and patched into the prototype definitions using a heuristic.
    pub fn new<'a>(
        factorio_binary: &'a Path,
        api: &'a Api,
        locale: &'a str,
        export_icons: bool,
    ) -> Result<FactorioExporter<'a>> {
        let temp_dir = tempfile::Builder::new().prefix(MOD_NAME).tempdir()?;
        let mod_controller = ModController::new(temp_dir.path().join(MODS_DIR));
        Ok(FactorioExporter {
            factorio_binary,
            api,
            locale,
            temp_dir,
            mod_controller,
            export_icons,
        })
    }
}

const ARGS: &[&str] = &["--config", CONFIG];

impl FactorioExporter<'_> {
    /// Export the prototype definitions from Factorio and partially deserialize
    /// them into a [`serde_yaml::Value`] object, which can easily deserialized
    /// of serialized into other data types further.
    ///
    /// This function executes Factorio twice, once to create a save file, and a
    /// second time to execute an exporter mod that does the heavy-lifting. The
    /// process uses a temporary directory, so that the main Factorio
    /// installation is not touched. Any existing Factorio configuration,
    /// including installed mods are therefore ignored.
    pub fn export(&self) -> Result<Value> {
        self.create_exec_dir()?;
        self.create_exporter_mod()?;

        info!("create an empty save file");
        self.run_factorio(&["--create", SAVE, "--mod-directory", "none"])?;

        info!("execute Factorio to export prototypes");
        #[rustfmt::skip]
        let output = self.run_factorio(&[
            "--benchmark", SAVE,
            "--benchmark-ticks", "1",
            "--benchmark-runs", "1",
            "--instrument-mod", MOD_NAME,
        ])?;

        info!("parse Factorio output");
        self.parse_output(&String::from_utf8_lossy(&output.stdout))
    }

    fn create_exec_dir(&self) -> Result<()> {
        let config = self.temp_dir.path().join(CONFIG);

        debug!("creating config file: {:?}", config);
        writedoc!(
            File::create(config)?,
            r#"
                [path]
                read-data=__PATH__executable__/../../data
                write-data=.
                [general]
                locale={}
            "#,
            self.locale
        )?;
        Ok(())
    }

    /// Install mods into the temporary execution directory before exporting.
    /// This allows exporting additional items, recipes, and all other changes
    /// that the mods make to be part of the export.
    ///
    /// No particular checks are made that the dependencies of the specified
    /// mods can be resolved. This is the responsibility of the caller.
    /// Otherwise Factorio will probably not start.
    ///
    /// # Arguments
    ///
    /// * `mods` - A list of file system paths that point to Factorio mods in
    ///   `.zip` format.
    pub fn install_mods<I, P>(&self, mods: I) -> Result<()>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        info!("installing mods");

        for m in mods {
            debug!("installing mod: {:?}", m.as_ref());
            self.mod_controller.add_mod(m.as_ref())?;
        }
        Ok(())
    }

    fn create_exporter_mod(&self) -> Result<()> {
        self.mod_controller
            .create_mod(
                ModManifestBuilder::default()
                    .name(MOD_NAME)
                    .version(MOD_VERSION)
                    .title("Factorio Exporter")
                    .author("Michael Forster <email@michael-forster.de")
                    .build()
                    .unwrap(),
            )?
            .add_file("export.lua", include_str!("../lua/export.lua"))?
            .add_file(
                "instrument-after-data.lua",
                include_str!("../lua/instrument-after-data.lua"),
            )?
            .add_file("instrument-control.lua", include_str!("../lua/instrument-control.lua"))?
            .add_file("prototypes.lua", &ScriptGenerator::new(self.api).generate("game"))?;

        Ok(())
    }

    fn run_factorio(&self, args: &[&str]) -> Result<Output> {
        if !self.factorio_binary.is_file() {
            return Err(FactorioExporterError::FileNotFoundError {
                file: self.factorio_binary.into(),
            });
        }

        let mut binary = Command::new(self.factorio_binary);
        let command = &mut binary.current_dir(&self.temp_dir).args(ARGS).args(args);

        debug!("executing command: {:?}", command);

        let output = command.output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            error!("STDOUT\n{}", &stdout);
            error!("STDERR\n{}", &stderr);

            return Err(FactorioExecutionError { stdout, stderr });
        }

        Ok(output)
    }

    fn find_section<'a>(output: &'a str, marker: &str) -> Result<&'a str> {
        let start_marker = format!("<{marker}>");
        let start = output.find(&start_marker).ok_or_else(|| {
            FactorioExporterError::FactorioOutputError {
                message: format!("Didn't find {start_marker} marker"),
                output: output.into(),
            }
        })?;

        let stop_marker = format!("</{marker}>");
        let stop = output.find(&stop_marker).ok_or_else(|| {
            FactorioExporterError::FactorioOutputError {
                message: format!("Didn't find {stop_marker} marker"),
                output: output.into(),
            }
        })?;

        Ok(&output[start + start_marker.len()..stop])
    }

    fn parse_output(&self, s: &str) -> Result<Value> {
        // Unfortunately we have no control over the string printed by Lua's
        // `localised_print`. There can be single/double quotes or new lines in
        // there. Neither JSON nor YAML can deal with that well. YAML could if we
        // had a way to control the indentation, but we don't. So, let's solve it
        // the hacky way: post-processing.

        debug!("parse prototype output");
        let re = regex!(r"(?s)<STRING>(.*?)</STRING>");
        let sanitized = re.replace_all(Self::find_section(s, "EXPORT")?, |caps: &Captures| {
            format!("'{}'", &caps[1].replace('\n', "\\n").replace('\'', "''"))
        });
        let mut data: Value = serde_yaml::from_str::<Value>(&sanitized)?;

        debug!(
            "found {} items, {} recipes, {} technologies",
            data["item_prototypes"].as_mapping().unwrap().len(),
            data["recipe_prototypes"].as_mapping().unwrap().len(),
            data["technology_prototypes"].as_mapping().unwrap().len()
        );

        // Icon paths are not available in Factorio's runtime stage, so we must
        // resort to getting them in the data stage. Unfortunately data
        // structures in the data stage are a bit messy, so we need to apply
        // some heuristics to map icons into the prototypes that we get in the
        // runtime stage. We add an icon property to a prototype if it's `name`
        // and `object_name` or `type` match the section names and section
        // element names in `data.raw`.

        if self.export_icons {
            debug!("parse icons output");

            let icons: Vec<Icon> = serde_yaml::from_str(Self::find_section(s, "ICONS")?)?;
            let icons: HashMap<(&str, &str), &str> =
                icons.iter().map(|icon| ((icon.name, icon.section), icon.path)).collect();

            debug!("patch {} icons into prototypes", icons.len());

            let object_name_pattern = regex!("Lua(.*)Prototype");
            for (_, section) in data.as_mapping_mut().expect("root should be a mapping") {
                if let Value::Mapping(section) = section {
                    for (name, el) in section {
                        let name = name.as_str().expect("key should be a string");
                        let val = el.as_mapping_mut().expect("value should be mapping");

                        if let Some(path) = val
                            .get("type")
                            .and_then(|value| value.as_str())
                            .and_then(|ty| icons.get(&(name, ty)).map(|r| r.to_string()))
                            .or_else(|| {
                                val.get("object_name")
                                    .and_then(|value| value.as_str())
                                    .and_then(|name| {
                                        object_name_pattern
                                            .captures(name)
                                            .map(|captures| (&captures[1]).to_case(Case::Kebab))
                                    })
                                    .and_then(|ty| icons.get(&(name, &ty)).map(|r| r.to_string()))
                            })
                        {
                            val.insert("icon".into(), (*path).into());
                        };
                    }
                }
            }
        }

        Ok(data)
    }
}

#[derive(Debug, Deserialize)]
struct Icon<'a> {
    section: &'a str,
    name: &'a str,
    path: &'a str,
}
