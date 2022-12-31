use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use color_eyre::Result;
use serde::Deserialize;

use crate::App;

/// Load and analyze the recipe graph.
#[derive(Debug, Parser)]
pub struct AnalyzeCommand {
    prototypes: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub name: String,
}
#[derive(Debug, Deserialize)]
pub struct Item {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct Prototypes {
    entity_prototypes: HashMap<String, Entity>,
    recipe_prototypes: HashMap<String, Recipe>,
    item_prototypes: HashMap<String, Recipe>,
}

impl AnalyzeCommand {
    pub async fn execute(&self, app: &App) -> Result<()> {
        let prototypes: Prototypes =
            serde_json::from_reader(BufReader::new(File::open(&self.prototypes)?))?;

        dbg!(
            &prototypes.entity_prototypes["assembling-machine-1"],
            &prototypes.recipe_prototypes["assembling-machine-1"],
            &prototypes.item_prototypes["assembling-machine-1"]
        );
        Ok(())
    }
}
