use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::{Command, Output},
};

use indoc::writedoc;
use serde_json::Value;
use tempfile::TempDir;
use tracing::{debug, error, info};

use crate::{
    internal::mod_controller::ModController,
    FactorioExporterError::{self, FactorioExecutionError},
    Result,
};

const CONFIG: &str = "config.ini";
const MOD_NAME: &str = "factorio_exporter";
const MODS_DIR: &str = "mods";

/// Main class for orchestrating the export.
pub struct FactorioExporter<'a> {
    factorio_binary: &'a Path,
    locale: &'a str,
    temp_dir: TempDir,
    mod_controller: ModController,
}

impl FactorioExporter<'_> {
    /// Creates and configures a new `FactorioExporter` instance.
    ///
    /// # Arguments
    ///
    /// * `factorio_binary` - File system path of a Factorio binary. This can be
    ///   any variant of the binary, full, headless, or demo.
    /// * `locale` - Locale code to use for translated strings.
    pub fn new<'a>(factorio_binary: &'a Path, locale: &'a str) -> Result<FactorioExporter<'a>> {
        let temp_dir = tempfile::Builder::new().prefix(MOD_NAME).tempdir()?;
        let mod_controller = ModController::new(temp_dir.path().join(MODS_DIR));
        Ok(FactorioExporter { factorio_binary, locale, temp_dir, mod_controller })
    }
}

const ARGS: &[&str] = &["--config", CONFIG];

impl FactorioExporter<'_> {
    /// Export the prototype definitions from Factorio and partially deserialize
    /// them into a [`serde_yaml::Value`] object, which can easily deserialized
    /// of serialized into other data types further.
    pub fn export(&self) -> Result<Value> {
        self.create_exec_dir()?;

        info!("create an empty save file");
        self.run_factorio(&["--dump-data"])?;

        Ok(serde_json::from_slice(&fs::read(
            self.temp_dir.path().join("script-output/data-raw-dump.json"),
        )?)?)
    }

    fn create_exec_dir(&self) -> Result<()> {
        let config = self.temp_dir.path().join(CONFIG);
        std::fs::create_dir(self.temp_dir.path().join("script-output"))?;

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
}
