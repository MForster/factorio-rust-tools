use std::{collections::HashMap, fs, path::Path};

use itertools::Itertools;
use semver::Version;
use serde_derive::Deserialize;
use tracing::debug;

use crate::{FactorioExporterError, Result};

/// Loads the API definition from the definition file
///
/// This file is usually called `runtime-api.json` and can be found in the
/// `factorio/doc-html` directory of a full (not headless or demo) Factorio
/// installation, or on the [API website]. Its content can be browsed in a [nice
/// format] as well.
///
/// [API website]: https://lua-api.factorio.com/latest/runtime-api.json
/// [nice format]: https://lua-api.factorio.com/
///
/// # Arguments
///
/// * `api_file_path` - File system path to read the API definition from.
///
/// # Example
///
/// ```no_run
/// use factorio_exporter::{ load_api, FactorioExporterError };
/// use std::path::PathBuf;
///
/// let api_spec = PathBuf::from("/home/user/factorio/doc-html/runtime-api.json");
/// let api = load_api(&api_spec)?;
///
/// # Ok::<(), FactorioExporterError>(())
/// ```
pub fn load_api(api_file_path: &Path) -> Result<Api> {
    debug!("Loading API definition file from {}", &api_file_path.display());

    if !api_file_path.is_file() {
        return Err(FactorioExporterError::FileNotFoundError { file: api_file_path.into() });
    }

    let s = fs::read_to_string(api_file_path)?;
    let api: Api = serde_json::from_str(&s)?;

    debug!("parsed API, got {} classes and {} concepts", &api.classes.len(), &api.concepts.len());
    Ok(api)
}

/// Meta description of the factorio API.
///
/// [Documentation](https://lua-api.factorio.com/latest/json-docs.html)
/// [JSON data](https://lua-api.factorio.com/latest/runtime-api.json)
///
/// This is for API version 3, introduced with Factorio 1.1.62
#[derive(Debug, Deserialize)]
#[serde(from = "RawApi")]
pub struct Api {
    pub application_version: Version,
    pub classes: HashMap<String, Class>,
    pub concepts: HashMap<String, Concept>,
}

impl Api {
    pub fn toplevel_attributes(&self) -> Vec<&Attribute> {
        self.classes["LuaGameScript"]
            .attributes()
            .iter()
            .copied()
            .sorted_by_key(|attr| attr.order)
            .filter(|attr| attr.name.ends_with("_prototypes"))
            .collect_vec()
    }
}

#[derive(Debug, Deserialize)]
struct RawApi {
    application: String,
    application_version: Version,
    api_version: i32,
    stage: String,
    classes: Vec<Class>,
    concepts: Vec<Concept>,
}

impl From<RawApi> for Api {
    fn from(raw: RawApi) -> Self {
        assert!(raw.application == "factorio");
        assert!(raw.api_version == 3);
        assert!(raw.stage == "runtime");

        Api {
            application_version: raw.application_version,
            classes: raw.classes.into_iter().map(|class| (class.name.clone(), class)).collect(),
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
    pub description: String,
    pub notes: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
    pub order: u64,
    pub base_classes: Option<Vec<String>>,
}

pub trait HasAttributes {
    fn attributes(&self) -> Vec<&Attribute>;
}

impl HasAttributes for Class {
    fn attributes(&self) -> Vec<&Attribute> {
        self.attributes
            .values()
            // TODO: Support subclasses and base classes
            .filter(|a| a.read != Some(false) && a.subclasses.is_none())
            .sorted_by_key(|attr| &attr.order)
            .collect_vec()
    }
}

#[derive(Debug, Deserialize)]
struct RawClass {
    name: String,
    attributes: Vec<Attribute>,
    description: String,
    notes: Option<Vec<String>>,
    examples: Option<Vec<String>>,
    order: u64,
    base_classes: Option<Vec<String>>,
}

impl From<RawClass> for Class {
    fn from(raw: RawClass) -> Self {
        Class {
            name: raw.name,
            attributes: raw.attributes.into_iter().map(|attr| (attr.name.clone(), attr)).collect(),
            description: raw.description,
            notes: raw.notes,
            examples: raw.examples,
            order: raw.order,
            base_classes: raw.base_classes,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub r#type: Type,
    pub optional: bool,
    pub order: u64,
    pub description: String,
    pub read: Option<bool>,
    pub subclasses: Option<Vec<String>>,
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
    pub r#type: Type,
    pub description: String,
    pub notes: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
    pub order: u64,
}

impl HasAttributes for Concept {
    fn attributes(&self) -> Vec<&Attribute> {
        if let Type::Table { parameters, variant_parameter_groups } = &self.r#type {
            variant_parameter_groups
                .iter()
                .flatten()
                .flat_map(|group| &group.parameters)
                .chain(parameters)
                .sorted_by_key(|&a| &a.name)
                .dedup_by(|a, b| a.name == b.name)
                .sorted_by_key(|&a| &a.order)
                .collect_vec()
        } else {
            Vec::new()
        }
    }
}

pub fn is_number(r#type: &Type) -> bool {
    use self::Type::*;
    match r#type {
        Int8 | Int | UInt8 | UInt16 | UInt | UInt64 | Double | Float | Number => true,
        Union { options } => options.iter().all(is_number),
        _ => false,
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "RawType")]
pub enum Type {
    Int8,
    Int,
    UInt8,
    UInt16,
    UInt,
    UInt64,
    Double,
    Float,
    Number,
    String,
    Boolean,

    NamedType {
        name: String,
    },

    Array {
        value: Box<Type>,
    },

    Dictionary {
        key: Box<Type>,
        value: Box<Type>,
    },

    Literal {
        value: LiteralValue,
        description: Option<String>,
    },

    LuaCustomTable {
        key: Box<Type>,
        value: Box<Type>,
    },

    Struct {
        attributes: Vec<Attribute>,
    },

    Table {
        parameters: Vec<Attribute>,
        variant_parameter_groups: Option<Vec<VariantParameterGroup>>,
    },

    Tuple {
        parameters: Vec<Attribute>,
    },

    Type {
        value: Box<Type>,
    },

    Union {
        options: Vec<Type>,
    },
}

impl<'a> From<RawType<'a>> for Type {
    fn from(raw: RawType<'a>) -> Self {
        use ComplexType::*;
        use RawType::*;
        match raw {
            String("int8") => Self::Int8,
            String("int") => Self::Int,
            String("uint8") => Self::UInt8,
            String("uint16") => Self::UInt16,
            String("uint") => Self::UInt,
            String("uint64") => Self::UInt64,
            String("double") => Self::Double,
            String("float") => Self::Float,
            String("number") => Self::Number,
            String("string" | "LocalisedString") => Self::String,
            String("boolean") => Self::Boolean,

            String(name) => Self::NamedType { name: name.into() },

            Complex(Array { value }) => Self::Array { value },
            Complex(Dictionary { key, value }) => Self::Dictionary { key, value },
            Complex(Literal { value, description }) => Self::Literal { value, description },
            Complex(LuaCustomTable { key, value }) => Self::LuaCustomTable { key, value },
            Complex(Struct { attributes }) => Self::Struct { attributes },
            Complex(Table { parameters, variant_parameter_groups }) => {
                Self::Table { parameters, variant_parameter_groups }
            }
            Complex(Tuple { parameters }) => Self::Tuple { parameters },
            Complex(Type { value }) => Self::Type { value },
            Complex(Union { options }) => Self::Union { options },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawType<'a> {
    String(&'a str),
    Complex(ComplexType),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "complex_type")]
#[serde(rename_all = "lowercase")]
enum ComplexType {
    Array {
        value: Box<Type>,
    },

    Dictionary {
        key: Box<Type>,
        value: Box<Type>,
    },

    Literal {
        value: LiteralValue,
        description: Option<String>,
    },

    #[serde(rename = "LuaCustomTable")]
    LuaCustomTable {
        key: Box<Type>,
        value: Box<Type>,
    },

    Struct {
        attributes: Vec<Attribute>,
    },

    Table {
        parameters: Vec<Attribute>,
        variant_parameter_groups: Option<Vec<VariantParameterGroup>>,
    },

    Tuple {
        parameters: Vec<Attribute>,
    },

    Type {
        value: Box<Type>,
    },

    Union {
        options: Vec<Type>,
    },
}
