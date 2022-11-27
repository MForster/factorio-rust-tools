use std::collections::HashMap;

use convert_case::Case::UpperCamel;
use convert_case::Casing;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use syn::Ident;

use crate::api::{is_number, Concept, HasAttributes};
use crate::api::{Api, Attribute, Class, Type};

fn id(name: &str) -> Ident {
    syn::parse_str(name).unwrap_or_else(|_| format_ident!("r#{}", name))
}

pub struct ApiGenerator<'a> {
    api: &'a Api,
    generated_structs: HashMap<&'a str, TokenStream>,
}

impl<'a> ApiGenerator<'a> {
    pub fn new(api: &'a Api) -> ApiGenerator<'a> {
        ApiGenerator {
            api,
            generated_structs: HashMap::new(),
        }
    }

    pub fn generate_api(mut self) -> String {
        let mut toplevel_attrs = Vec::new();

        for (class, attribute) in self.api.classes["LuaGameScript"]
            .attributes()
            .iter()
            .filter_map(|attr| {
                if let Type::LuaCustomTable { value, .. } = &attr.r#type {
                    if let Type::NamedType { name } = value.as_ref() {
                        if attr.name.ends_with("prototypes") {
                            return Some((name, &attr.name));
                        }
                    }
                }
                None
            })
        {
            self.visit_concept_or_class(class);

            let name = id(attribute);
            let cls = id(class);
            toplevel_attrs.push(quote! {
                #name: HashMap<String, #cls>,
            });
        }

        let struct_code = self
            .generated_structs
            .iter()
            .sorted_by_key(|&(n, _)| n)
            .map(|(_, c)| c);

        quote! {
            use serde_derive::Deserialize;
            use std::collections::HashMap;

            _blank_!();
            #[derive(Debug,Deserialize)]
            pub struct PrototypeExport {
                #(pub #toplevel_attrs)*
            }

            #(#struct_code)*
        }
        .to_string()
    }

    fn visit_concept_or_class(&mut self, name: &'a str) -> bool {
        if self.generated_structs.contains_key(name) {
            return true;
        } else if self.api.classes.contains_key(name) {
            self.generated_structs.insert(name, quote!());
            let v = self.create_struct_code(&self.api.classes[name]);
            self.generated_structs.insert(name, v);
            return true;
        } else if self.api.concepts.contains_key(name) {
            self.generated_structs.insert(name, quote!());
            let v = self.create_concept_code(&self.api.concepts[name]);
            self.generated_structs.insert(name, v);
            return true;
        }
        false
    }

    fn doc_comment(text: &str) -> TokenStream {
        if text.is_empty() {
            quote! {}
        } else {
            let text = format!(" {}", &text);
            quote! {
                #[doc=#text]
            }
        }
    }

    fn create_concept_code(&mut self, concept: &'a Concept) -> TokenStream {
        let name = id(&concept.name);
        let mut attr_code = Vec::new();
        let mut enum_code = Vec::new();

        if let Type::Table {
            parameters,
            variant_parameter_groups,
        } = &concept.r#type
        {
            attr_code.extend(
                parameters
                    .iter()
                    .sorted_by_key(|a| &a.name)
                    .dedup_by(|a1, a2| a1.name == a2.name)
                    .sorted_by_key(|a| &a.order)
                    .flat_map(|a| self.create_attr_code(a, true)),
            );

            if let Some(group) = variant_parameter_groups {
                let variants_code = group.iter().sorted_by_key(|v| v.order).map(|variant| {
                    let variant_name = id(&variant.name.to_case(UpperCamel));
                    let variant_attr_code = variant
                        .parameters
                        .iter()
                        .map(|attr| self.create_attr_code(attr, false));

                    quote! {
                        #variant_name {
                            #(#variant_attr_code)*
                        }
                    }
                });

                let enum_name = format_ident!("{}Variant", concept.name);
                enum_code.push(quote! {
                    _blank_!();
                    #[derive(Debug, Deserialize)]
                    #[serde(untagged)]
                    pub enum #enum_name {
                        #(#variants_code,)*
                     }
                });
                attr_code.push(quote! {
                    #[serde(flatten)]
                    pub variant: #enum_name,
                });
            }
        }

        let doc_comment = ApiGenerator::doc_comment(&concept.description);
        quote! {
            _blank_!();
            #[derive(Debug, Deserialize)]
            #doc_comment
            pub struct #name{
                #(#attr_code)*
            }
            #(#enum_code)*
        }
    }

    fn create_struct_code(&mut self, class: &'a Class) -> TokenStream {
        let name = id(&class.name);

        let attrs = class.attributes();
        let attr_code = attrs.iter().filter_map(|a| self.create_attr_code(a, true));

        let doc_comment = ApiGenerator::doc_comment(&class.description);

        quote! {
            _blank_!();
            #[derive(Debug, Deserialize)]
            #doc_comment
            pub struct #name {
                #(#attr_code)*
            }
        }
    }

    fn create_attr_code(&mut self, attr: &'a Attribute, public: bool) -> Option<TokenStream> {
        let name = id(&attr.name);

        self.attr_type_code(attr).map(|mut ty| {
            let doc_comment = ApiGenerator::doc_comment(&attr.description);

            if attr.optional || attr.name == "items_to_place_this" {
                ty = quote! { Option<#ty> }
            }

            let public = if public {
                quote! {pub}
            } else {
                quote! {}
            };

            quote! {
                #doc_comment
                #public #name: #ty,
            }
        })
    }

    fn attr_type_code(&mut self, attr: &'a Attribute) -> Option<TokenStream> {
        use self::Type::*;
        match &attr.r#type {
            Int8 => Some(quote! {i8}),
            Int => Some(quote! {i32}),
            UInt8 => Some(quote! {u8}),
            UInt16 => Some(quote! {u16}),
            UInt => Some(quote! {u32}),
            UInt64 => Some(quote! {u64}),
            Double => Some(quote! {f32}),
            Float => Some(quote! {f64}),
            Number => Some(quote! {f64}),
            ty if is_number(ty) => Some(quote! {f64}),
            String => Some(quote! {String}),
            Boolean => Some(quote! {bool}),
            Array { value } => {
                if let NamedType { name } = value.as_ref() {
                    if name == "TriggerEffectItem" {
                        // TODO: Investigate why there seem to be instances of this
                        // concept that miss mandatory fields.
                        None
                    } else if self.api.concepts.contains_key(name) {
                        self.visit_concept_or_class(name);
                        let concept_name = id(name);
                        Some(quote! { Vec<#concept_name> })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
