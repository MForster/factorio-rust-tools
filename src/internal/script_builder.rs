use std::fmt::{self, Write};

use indoc::writedoc;

#[derive(Default)]
pub struct ScriptBuilder {
    script: String,
    indentation: usize,
}

impl ScriptBuilder {
    pub fn new() -> ScriptBuilder {
        let mut builder = ScriptBuilder::default();

        writedoc! {&mut builder, "
            local prototypes={{}}
            export = require('export')
            function prototypes.export()
        "}
        .unwrap();

        builder.indentation += 4;
        builder
    }

    pub fn export_primitive_value(&mut self, primitive_type: &str, value: &str) {
        writeln!(self, r#"export.Export{primitive_type}Value({value})"#,).unwrap();
    }

    pub fn export_string_value(&mut self, value: &str) {
        self.export_primitive_value("String", value);
    }

    pub fn export_number_value(&mut self, value: &str) {
        self.export_primitive_value("Number", value);
    }

    pub fn export_bool_value(&mut self, value: &str) {
        self.export_primitive_value("Bool", value);
    }

    pub fn export_primitive_attr(&mut self, primitive_type: &str, object: &str, property: &str) {
        writeln!(
            self,
            r#"export.Export{primitive_type}Attr("{property}", {object}.{property})"#,
        )
        .unwrap();
    }

    pub fn export_string_attr(&mut self, object: &str, property: &str) {
        self.export_primitive_attr("String", object, property);
    }

    pub fn export_number_attr(&mut self, object: &str, property: &str) {
        self.export_primitive_attr("Number", object, property);
    }

    pub fn export_bool_attr(&mut self, object: &str, property: &str) {
        self.export_primitive_attr("Bool", object, property);
    }

    pub fn begin_block(&mut self, block_type: &str, object: &str, attribute: &str) -> String {
        let element = "prototype";
        writeln!(
            self,
            r#"export.Export{block_type}("{attribute}", {object}, function({element})"#,
        )
        .unwrap();
        self.indentation += 4;
        element.into()
    }

    pub fn end_block(&mut self) {
        self.indentation -= 4;
        writeln!(self, "end)").unwrap();
    }

    pub fn begin_table(&mut self, table: &str, attribute: &str) -> String {
        self.begin_block("Table", table, attribute)
    }

    pub fn end_table(&mut self) {
        self.end_block();
    }

    pub fn begin_array(&mut self, array: &str, attribute: &str) -> String {
        self.begin_block("Array", &format!("{array}.{attribute}"), attribute)
    }

    pub fn end_array(&mut self) {
        self.end_block();
    }

    pub fn build(mut self) -> String {
        self.indentation -= 4;

        writeln!(self, r#"end"#).unwrap();
        writeln!(self, r#"return prototypes"#).unwrap();

        self.script
    }
}
impl Write for ScriptBuilder {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let indentation = if self.script.is_empty() || self.script.ends_with('\n') {
            self.indentation
        } else {
            0
        };
        self.script.push_str(&format!("{:indentation$}{s}", ""));
        Ok(())
    }
}
