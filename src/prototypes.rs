use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PrototypeExport {
    pub item_prototypes: HashMap<String, Item>,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name: String,
}
