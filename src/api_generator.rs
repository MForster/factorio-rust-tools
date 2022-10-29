use crate::{
    api::{Api, Attribute, ComplexType, Type},
    exporter_script_builder::ExporterScriptBuilder,
};

pub fn generate_exporter_script(api: &Api) -> String {
    let mut script = ExporterScriptBuilder::new();

    for (class, attribute) in
        api.classes["LuaGameScript"]
            .attributes
            .iter()
            .filter_map(|(_, attr)| {
                if let Type::Complex(ComplexType::LuaCustomTable { value, .. }) = &attr.ty {
                    if let Type::String(value_type) = &**value {
                        if value_type.ends_with("Prototype") {
                            return Some((value_type, &attr.name));
                        }
                    }
                }
                None
            })
    {
        let object = script.begin_table(&format!("game.{attribute}"), attribute);

        for attr in api.classes[class].attributes.values() {
            visit_attribute(attr, &mut script, api, &object);
        }
        script.end_table();
    }

    script.build()
}

fn is_number(ty: &Type) -> bool {
    match ty {
        Type::String(s) if s == "double" || s == "uint" => true,
        Type::Complex(ComplexType::Union { options }) if options.iter().all(is_number) => true,
        _ => false,
    }
}

fn visit_attribute(attr: &Attribute, script: &mut ExporterScriptBuilder, api: &Api, object: &str) {
    match &attr.ty {
        Type::String(s) if s == "string" || s == "LocalisedString" => {
            script.export_string(object, &attr.name);
        }

        ty if is_number(ty) => {
            script.export_number(object, &attr.name);
        }

        Type::String(s) if s == "boolean" => {
            script.export_bool(object, &attr.name);
        }

        Type::Complex(ComplexType::Array { value }) => {
            if let Type::String(s) = &**value {
                if let Some(concept) = api.concepts.get(s) {
                    if let Type::Complex(ComplexType::Table {
                        parameters,
                        variant_parameter_groups,
                    }) = &concept.ty
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

        Type::Complex(ComplexType::Dictionary { value, .. }) => {
            if let Type::String(s) = &**value {
                if let Some(class) = api.classes.get(s) {
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
