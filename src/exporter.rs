use std::{
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use indoc::writedoc;

use itertools::Itertools;
use regex::{Captures, Regex};
use tempfile::TempDir;
use tracing::{debug, error, info};

use crate::{
    api::{Api, HasAttributes},
    internal::{
        mod_controller::{ModController, ModManifestBuilder},
        script_generator::ScriptGenerator,
    },
    FactorioExporterError::FactorioExecutionError,
    Result,
};

const CONFIG: &str = "config.ini";
const SAVE: &str = "save.zip";
const MOD_NAME: &str = "factorio_exporter";
const MOD_VERSION: &str = "0.0.1";
const MODS_DIR: &str = "mods";

pub struct FactorioExporter<'a> {
    factorio_dir: &'a Path,
    api: &'a Api,
    locale: &'a str,
    temp_dir: TempDir,
}

impl FactorioExporter<'_> {
    pub fn new<'a>(
        factorio_dir: &'a Path,
        api: &'a Api,
        locale: &'a str,
    ) -> Result<FactorioExporter<'a>> {
        Ok(FactorioExporter {
            factorio_dir,
            api,
            locale,
            temp_dir: tempfile::Builder::new().prefix(MOD_NAME).tempdir()?,
        })
    }
}

const FACTORIO_BINPATH: &str = "bin/x64/factorio";
const ARGS: &[&str] = &["--config", CONFIG, "--mod-directory", MODS_DIR];

impl FactorioExporter<'_> {
    pub fn export(&self) -> Result<String> {
        self.create_exec_dir()?;
        self.create_exporter_mod()?;

        info!("create an empty save file");
        self.run_factorio(&["--create", SAVE])?;

        info!("execute Factorio to export prototypes");
        #[rustfmt::skip]
        let output = self.run_factorio(&[
            "--benchmark", SAVE,
            "--benchmark-ticks", "1",
            "--benchmark-runs", "1",
            "--instrument-mod", MOD_NAME,
        ])?;

        self.parse_output(&String::from_utf8_lossy(&output.stdout))
    }

    pub fn json(&self) -> Result<String> {
        let json: serde_json::Value = serde_yaml::from_str(&self.export()?)?;
        Ok(json.to_string())
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

    fn create_exporter_mod(&self) -> Result<()> {
        let attrs = self.api.classes["LuaGameScript"]
            .attributes()
            .iter()
            .copied()
            .filter(|attr| attr.name.ends_with("prototypes"))
            .collect_vec();

        ModController::new(self.temp_dir.path().join(MODS_DIR))
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
                "instrument-control.lua",
                include_str!("../lua/instrument-control.lua"),
            )?
            .add_file(
                "prototypes.lua",
                &ScriptGenerator::new(self.api).generate("game", attrs),
            )?;

        Ok(())
    }

    fn run_factorio(&self, args: &[&str]) -> Result<Output> {
        let mut binary = Command::new(self.factorio_dir.join(FACTORIO_BINPATH));
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

    fn parse_output(&self, s: &str) -> Result<String> {
        let begin = s.find("<EXPORT>").expect("Didn't find <EXPORT> marker");
        let end = s.find("</EXPORT>").expect("Didn't find </EXPORT> marker");
        let yaml = &s[begin + 8..end];

        // Unfortunately we have no control over the string printed by Lua's
        // `localised_print`. There can be single/double quotes or new lines in
        // there. Neither JSON nor YAML can deal with that well. YAML could if we
        // had a way to control the indentation, but we don't. So, let's solve it
        // the hacky way: post-processing.
        let re = Regex::new(r"(?s)<STRING>(.*?)</STRING>").unwrap();
        Ok(re
            .replace_all(yaml, |caps: &Captures| {
                format!("'{}'", &caps[1].replace('\n', "\\n").replace('\'', "''"))
            })
            .into())
    }
}
