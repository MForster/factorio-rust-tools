use std::collections::HashSet;

use itertools::Itertools;

use crate::api::{is_number, HasAttributes};
use crate::api::{Api, Attribute, Type};

use super::script_builder::ScriptBuilder;

pub struct ScriptGenerator<'a> {
    api: &'a Api,
    script: ScriptBuilder,
    toplevel_classes: HashSet<&'a str>,
}

impl<'a> ScriptGenerator<'a> {
    pub fn new(api: &'a Api) -> ScriptGenerator<'a> {
        let toplevel_classes = api.classes["LuaGameScript"]
            .attributes()
            .iter()
            .filter_map(|attr| {
                if attr.name.ends_with("_prototypes") {
                    if let Type::LuaCustomTable { value, .. } = &attr.r#type {
                        if let Type::NamedType { name } = value.as_ref() {
                            return Some(name.as_str());
                        }
                    }
                }
                None
            })
            .collect();

        let script = ScriptBuilder::new();
        ScriptGenerator { api, script, toplevel_classes }
    }

    pub fn generate(mut self, object: &str, attributes: Vec<&Attribute>) -> String {
        self.script.begin_context(object);
        self.export_attrs(attributes, 0);
        self.script.end_block();
        self.script.build()
    }

    fn export_attrs(&mut self, attrs: Vec<&Attribute>, depth: usize) {
        for attr in attrs {
            match &attr.r#type {
                Type::String => self.script.export_string_attr(&attr.name),
                Type::Boolean => self.script.export_bool_attr(&attr.name),
                ty if is_number(ty) => self.script.export_number_attr(&attr.name),

                _ => {
                    self.script.begin_object(&attr.name);
                    self.export_value(&attr.r#type, depth);
                    self.script.end_block();
                }
            }
        }
    }

    fn export_value(&mut self, ty: &Type, depth: usize) {
        let depth = depth + 1;

        match ty {
            Type::NamedType { name } => {
                if let Some(class) = self.api.classes.get(name) {
                    let mut attrs = class.attributes();

                    // To avoid infinite loops in the object graph, we cut the
                    // recursive visitation as soon as we get to a class that is
                    // stored in one of the top-level tables.
                    if depth > 2 && self.toplevel_classes.contains(&**name) {
                        attrs.retain(|a| a.name == "name");
                        assert!(!attrs.is_empty(),
                            "don't know how to reference top-level classes without a `name` attribute: {name}");
                    }

                    self.export_attrs(attrs, depth);
                }

                if let Some(concept) = self.api.concepts.get(name) {
                    self.export_attrs(concept.attributes(), depth);
                }
            }

            Type::LuaCustomTable { key, value } | Type::Dictionary { key, value }
                if matches!(key.as_ref(), Type::String) =>
            {
                self.script.begin_mapping();
                self.export_value(value, depth);
                self.script.end_block();
            }

            Type::Array { value } => {
                self.script.begin_array();
                self.export_value(value, depth);
                self.script.end_block();
            }

            Type::Table { parameters, variant_parameter_groups } => {
                self.export_attrs(
                    variant_parameter_groups
                        .iter()
                        .flatten()
                        .flat_map(|vpg| &vpg.parameters)
                        .chain(parameters)
                        .collect_vec(),
                    depth,
                );
            }

            Type::String => self.script.export_string_value(),
            Type::Boolean => self.script.export_bool_value(),
            ty if is_number(ty) => self.script.export_number_value(),

            Type::Union { .. } => {} // TODO

            _ => unimplemented!("unsupported attribute type: {ty:#?}"),
        };
    }
}
