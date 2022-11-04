use itertools::Itertools;

use crate::api::{is_number, HasAttributes};
use crate::api::{Api, Attribute, Type};

use super::script_builder::ScriptBuilder;

pub struct ScriptGenerator<'a> {
    api: &'a Api,
    script: ScriptBuilder,
}

impl<'a> ScriptGenerator<'a> {
    pub fn new(api: &'a Api) -> ScriptGenerator<'a> {
        let script = ScriptBuilder::new();
        ScriptGenerator { api, script }
    }

    pub fn generate(mut self, object: &str, attributes: Vec<&Attribute>) -> String {
        self.script.begin_context(object);
        self.export_attrs(attributes, 0);
        self.script.end_block();
        self.script.build()
    }

    fn export_attrs(&mut self, attrs: Vec<&Attribute>, depth: usize) {
        for attr in attrs {
            // TODO: Reading fails, but the documentation only says so in prose.
            if attr.name == "order_in_recipe" || attr.name == "subgroups" || attr.name == "group" {
                continue;
            }

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
                    // TODO: Cut infinite recursion in a more principled way
                    if depth > 2 {
                        attrs.retain(|a| a.name == "name")
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

            Type::Table {
                parameters,
                variant_parameter_groups,
            } => {
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
