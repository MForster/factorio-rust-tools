use std::collections::HashMap;

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::api::{Api, Type};

pub struct ApiGenerator<'a> {
    api: &'a Api,
}
fn id(name: &str) -> Ident {
    syn::parse_str(name).unwrap_or_else(|_| format_ident!("r#{}", name))
}
impl<'a> ApiGenerator<'a> {
    pub fn new(api: &'a Api) -> ApiGenerator<'a> {
        ApiGenerator { api }
    }

    pub fn generate(&self) -> TokenStream {
        let mut attr_code = Vec::new();
        let mut type_code = HashMap::new();

        for attr in self.api.toplevel_attributes() {
            if let Type::LuaCustomTable { value, .. } = &attr.r#type {
                if let Type::NamedType { name: ty } = value.as_ref() {
                    if !type_code.contains_key(ty) {
                        type_code.insert(ty, self.generate_type(ty));
                    }

                    let name = id(&attr.name);
                    let ty = id(ty);
                    attr_code.push(quote! { pub #name: Option<HashMap<String, #ty>>, });
                }
            }
        }

        let type_code =
            type_code.into_iter().sorted_by_key(|(k, _)| *k).map(|(_, v)| v).collect_vec();
        quote! {
            use std::collections::HashMap;
            use serde::Deserialize;
            #(#type_code)*
            #[derive(Debug,Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct Prototypes {
                #(#attr_code)*
            }
        }
    }

    fn generate_type(&self, name: &str) -> TokenStream {
        let Some(class) = self.api.classes.get(name) else { return quote! {} };
        let class_name = id(&class.name);

        let mut table_code = Vec::new();
        let mut attr_code = Vec::new();
        for attr in class.attributes.values().sorted_by_key(|a| &a.name) {
            let attr_name = id(&attr.name);
            let attr_type = match &attr.r#type {
                Type::Boolean => Some(quote! { bool }),
                Type::Double => Some(quote! { f64 }),
                Type::Float => Some(quote! { f32 }),
                Type::Int => Some(quote! { i32 }),
                Type::Int8 => Some(quote! { i8 }),
                Type::Number => Some(quote! { f64 }),
                Type::String => Some(quote! { String }),
                Type::UInt => Some(quote! { u32 }),
                Type::UInt16 => Some(quote! { u16 }),
                Type::UInt64 => Some(quote! { u64 }),
                Type::UInt8 => Some(quote! { u8 }),
                Type::Table { .. } => {
                    let table_name = id(&format!("{name}{}", &attr.name.to_case(Case::UpperCamel)));

                    table_code.push(quote! {
                        #[derive(Debug, Deserialize)]
                        pub struct #table_name {

                        }
                    });

                    Some(quote! { #table_name })
                }
                Type::Array { value } => Some(quote! { Vec<()> }),
                _ => None,
            };
            if let Some(attr_type) = attr_type {
                attr_code.push(quote! { pub #attr_name: #attr_type, });
            }
        }

        quote! {
            #(#table_code)*
            #[derive(Debug,Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct #class_name {
                #(#attr_code)*
            }
        }
    }
}
