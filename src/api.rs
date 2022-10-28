use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(from = "RawApi")]
pub struct Api {
    pub classes: HashMap<String, Class>,
    pub concepts: HashMap<String, Concept>,
}

#[derive(Debug, Deserialize)]
struct RawApi {
    classes: Vec<Class>,
    concepts: Vec<Concept>,
}

impl From<RawApi> for Api {
    fn from(raw: RawApi) -> Self {
        Api {
            classes: raw
                .classes
                .into_iter()
                .map(|class| (class.name.clone(), class))
                .collect(),
            concepts: raw
                .concepts
                .into_iter()
                .map(|concept| (concept.name.clone(), concept))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "RawClass")]
pub struct Class {
    pub name: String,
    pub attributes: HashMap<String, Attribute>,
}

#[derive(Debug, Deserialize)]
struct RawClass {
    name: String,
    attributes: Vec<Attribute>,
}

impl From<RawClass> for Class {
    fn from(raw: RawClass) -> Self {
        Class {
            name: raw.name,
            attributes: raw
                .attributes
                .into_iter()
                .map(|attr| (attr.name.clone(), attr))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Type {
    String(String),
    Boolean(bool),
    Complex(ComplexType),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "complex_type")]
pub enum ComplexType {
    #[serde(rename = "array")]
    Array {
        value: Box<Type>,
    },

    #[serde(rename = "dictionary")]
    Dictionary {
        key: Box<Type>,
        value: Box<Type>,
    },

    #[serde(rename = "literal")]
    Literal {
        value: LiteralValue,
        description: Option<String>,
    },

    LuaCustomTable {
        key: Box<Type>,
        value: Box<Type>,
    },

    #[serde(rename = "struct")]
    Struct {
        attributes: Vec<Attribute>,
    },

    #[serde(rename = "table")]
    Table {
        parameters: Vec<Attribute>,
        variant_parameter_groups: Option<Vec<VariantParameterGroup>>,
    },

    #[serde(rename = "tuple")]
    Tuple {
        parameters: Vec<Attribute>,
    },

    #[serde(rename = "type")]
    Type {
        value: Box<Type>,
    },

    #[serde(rename = "union")]
    Union {
        options: Vec<Type>,
    },
}
#[derive(Debug, Deserialize)]
pub struct VariantParameterGroup {
    pub name: String,
    pub order: u64,
    pub parameters: Vec<Attribute>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LiteralValue {
    String(String),
    Boolean(bool),
}

#[derive(Debug, Deserialize)]
pub struct Concept {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}
