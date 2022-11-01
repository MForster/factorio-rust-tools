use crate::{
    api::{is_number, Api, Attribute, HasAttributes, Type},
    exporter_script_builder::ExporterScriptBuilder,
};

pub fn generate_exporter_script(api: &Api) -> String {
    let mut script = ExporterScriptBuilder::new();

    for (class, attribute) in
        api.classes["LuaGameScript"]
            .attributes
            .iter()
            .filter_map(|(_, attr)| {
                if let Type::LuaCustomTable { value, .. } = &attr.r#type {
                    if let Type::NamedType { name } = &**value {
                        if name.ends_with("Prototype") {
                            return Some((name, &attr.name));
                        }
                    }
                }
                None
            })
    {
        let object = script.begin_table(&format!("game.{attribute}"), attribute);

        for attr in api.classes[class].attributes() {
            visit_attribute(attr, &mut script, api, &object);
        }
        script.end_table();
    }

    script.build()
}

fn visit_attribute(attr: &Attribute, script: &mut ExporterScriptBuilder, api: &Api, object: &str) {
    match &attr.r#type {
        Type::String => {
            script.export_string(object, &attr.name);
        }

        ty if is_number(ty) => {
            script.export_number(object, &attr.name);
        }

        Type::Boolean => {
            script.export_bool(object, &attr.name);
        }

        Type::Array { value } => {
            if let Type::NamedType { name } = &**value {
                if let Some(concept) = api.concepts.get(name) {
                    if let Type::Table {
                        parameters,
                        variant_parameter_groups,
                    } = &concept.r#type
                    {
                        let element = script.begin_array(object, &attr.name);

                        for parameter in parameters {
                            visit_attribute(parameter, script, api, &element);
                        }

                        for group in variant_parameter_groups.iter().flatten() {
                            for parameter in &group.parameters {
                                visit_attribute(parameter, script, api, &element);
                            }
                        }

                        script.end_array();
                    }
                }
            }
        }

        Type::Dictionary { value, .. } => {
            if let Type::NamedType { name } = &**value {
                if let Some(class) = api.classes.get(name) {
                    let element = script.begin_array(object, &attr.name);

                    for attr in class.attributes.values() {
                        // TODO: Cut infinite loop in a more principled way
                        if attr.name == "name" {
                            visit_attribute(attr, script, api, &element);
                        }
                    }

                    script.end_array();
                }
            }
        }
        _ => {}
    }
}
