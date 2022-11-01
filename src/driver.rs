use std::{
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Output},
};

use indoc::writedoc;

use itertools::Itertools;
use regex::{Captures, Regex};
use tempdir::TempDir;
use tracing::{debug, error, info};

use crate::{
    api::{Api, HasAttributes},
    errors::{FactorioExporterError::FactorioExecutionError, Result},
    mod_controller::{ModController, ModManifestBuilder},
    prototypes::PrototypeExport,
    script_generator::ScriptGenerator,
};

const CONFIG: &str = "config.ini";
const SAVE: &str = "save.zip";
const MOD_NAME: &str = "factorio_exporter";
const MOD_VERSION: &str = "0.0.1";
const MODS_DIR: &str = "mods";

pub fn export(factorio_dir: &Path, api: &Api, locale: &str) -> Result<PrototypeExport> {
    let temp_dir = TempDir::new(MOD_NAME)?;

    create_exec_dir(temp_dir.path(), locale)?;
    create_exporter_mod(&temp_dir, api)?;

    info!("create an empty save file");
    run_factorio(factorio_dir, temp_dir.path(), &["--create", SAVE])?;

    info!("execute Factorio to export prototypes");
    #[rustfmt::skip]
    let output = run_factorio(factorio_dir, temp_dir.path(), &[
        "--benchmark", SAVE,
        "--benchmark-ticks", "1",
        "--benchmark-runs", "1",
        "--instrument-mod", MOD_NAME,
    ])?;

    parse_output(&String::from_utf8_lossy(&output.stdout))
}

fn create_exec_dir(exec_dir: &Path, locale: &str) -> Result<()> {
    let config = exec_dir.join(CONFIG);

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
        locale
    )?;
    Ok(())
}

fn create_exporter_mod(temp_dir: &TempDir, api: &Api) -> Result<()> {
    let attrs = api.classes["LuaGameScript"]
        .attributes()
        .iter()
        .copied()
        .filter(|attr| attr.name.ends_with("prototypes"))
        .collect_vec();

    ModController::new(temp_dir.path().join(MODS_DIR))
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
            &ScriptGenerator::new(api).generate("game", attrs),
        )?;

    Ok(())
}

const FACTORIO_BINPATH: &str = "bin/x64/factorio";
const ARGS: &[&str] = &["--config", CONFIG, "--mod-directory", MODS_DIR];

fn run_factorio(factorio_dir: &Path, exec_dir: &Path, args: &[&str]) -> Result<Output> {
    let mut binary = Command::new(factorio_dir.join(FACTORIO_BINPATH));
    let command = &mut binary.current_dir(exec_dir).args(ARGS).args(args);

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

fn parse_output(s: &str) -> Result<PrototypeExport> {
    let begin = s.find("<EXPORT>").expect("Didn't find <EXPORT> marker");
    let end = s.find("</EXPORT>").expect("Didn't find </EXPORT> marker");
    let yaml = &s[begin + 8..end];

    // Unfortunately we have no control over the string printed by Lua's
    // `localised_print`. There can be single/double quotes or new lines in
    // there. Neither JSON nor YAML can deal with that well. YAML could if we
    // had a way to control the indentation, but we don't. So, let's solve it
    // the hacky way: post-processing.
    let re = Regex::new(r"(?s)<STRING>(.*?)</STRING>").unwrap();
    let sanitized = re.replace_all(yaml, |caps: &Captures| {
        format!("'{}'", &caps[1].replace('\n', "\\n").replace('\'', "''"))
    });

    Ok(serde_yaml::from_str(&sanitized)?)
}
