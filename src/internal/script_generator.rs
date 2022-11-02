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
        self.export_attrs(object, attributes, 0);
        self.script.build()
    }

    fn export_attrs(&mut self, object: &str, attrs: Vec<&Attribute>, depth: usize) {
        for attr in attrs {
            // TODO: Reading fails, but the documentation only says so in prose.
            if attr.name == "order_in_recipe" || attr.name == "subgroups" {
                continue;
            }

            use self::Type::*;
            match &attr.r#type {
                String => {
                    self.script.export_string(object, &attr.name);
                }

                r#type if is_number(r#type) => {
                    self.script.export_number(object, &attr.name);
                }

                Boolean => {
                    self.script.export_bool(object, &attr.name);
                }

                Array { value } => {
                    let element = self.script.begin_array(object, &attr.name);
                    self.export_value(&element, value, depth);
                    self.script.end_array();
                }

                LuaCustomTable { value, .. } | Dictionary { value, .. } => {
                    let table = format!("{object}.{}", attr.name);
                    let element = self.script.begin_table(&table, &attr.name);
                    self.export_value(&element, value, depth);
                    self.script.end_table();
                }

                _ => {}
            }
        }
    }

    fn export_value(&mut self, object: &str, ty: &Type, depth: usize) {
        let depth = depth + 1;
        if let Type::NamedType { name } = ty {
            if let Some(class) = self.api.classes.get(name) {
                let mut attrs = class.attributes();
                // TODO: Cut infinite recursion in a more principled way
                if depth > 1 {
                    attrs.retain(|a| a.name == "name")
                }
                self.export_attrs(object, attrs, depth);
            }

            if let Some(concept) = self.api.concepts.get(name) {
                self.export_attrs(object, concept.attributes(), depth);
            }
        };
    }
}
