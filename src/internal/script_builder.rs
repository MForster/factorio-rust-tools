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

    pub fn export_string_value(&mut self) {
        writeln!(self, r#"export.ExportStringValue(context)"#).unwrap();
    }

    pub fn export_number_value(&mut self) {
        writeln!(self, r#"export.ExportNumberValue(context)"#).unwrap();
    }

    pub fn export_bool_value(&mut self) {
        writeln!(self, r#"export.ExportBoolValue(context)"#).unwrap();
    }

    pub fn export_string_attr(&mut self, attr: &str) {
        writeln!(self, r#"export.ExportStringAttr("{attr}", context.{attr})"#,).unwrap();
    }

    pub fn export_number_attr(&mut self, attr: &str) {
        writeln!(self, r#"export.ExportNumberAttr("{attr}", context.{attr})"#,).unwrap();
    }

    pub fn export_bool_attr(&mut self, attr: &str) {
        writeln!(self, r#"export.ExportBoolAttr("{attr}", context.{attr})"#,).unwrap();
    }

    pub fn begin_context(&mut self, context: &str) {
        writeln!(self, r#"export.SetContext({context}, function(context)"#,).unwrap();
        self.indentation += 4;
    }

    pub fn begin_object(&mut self, attr: &str) {
        writeln!(
            self,
            r#"export.ExportObject("{attr}", context.{attr}, function(context)"#,
        )
        .unwrap();
        self.indentation += 4;
    }

    pub fn begin_mapping(&mut self) {
        writeln!(self, r#"export.ExportMapping(context, function(context)"#,).unwrap();
        self.indentation += 4;
    }

    pub fn begin_array(&mut self) {
        writeln!(self, r#"export.ExportArray(context, function(context)"#,).unwrap();
        self.indentation += 4;
    }

    pub fn end_block(&mut self) {
        self.indentation -= 4;
        writeln!(self, "end)").unwrap();
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
