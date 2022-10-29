use std::fmt::{self, Write};

use indoc::writedoc;

#[derive(Default)]
pub struct ExporterScriptBuilder {
    script: String,
    indentation: usize,
}

impl ExporterScriptBuilder {
    pub fn new() -> ExporterScriptBuilder {
        let mut builder = ExporterScriptBuilder::default();

        writedoc! {&mut builder, "
            local prototypes={{}}
            export = require('export')
            function prototypes.export()
        "}
        .unwrap();

        builder.indentation += 4;
        builder
    }

    pub fn export_string(&mut self, object: &str, property: &str) {
        writeln!(
            self,
            r#"export.ExportString("{property}", {object}.{property})"#,
        )
        .unwrap();
    }

    pub fn begin_table(&mut self, table: &str, attribute: &str) -> String {
        let object = "prototype";
        writeln!(
            self,
            r#"export.ExportTable("{attribute}", {table}, function({object})"#,
        )
        .unwrap();
        self.indentation += 4;
        object.into()
    }

    pub fn end_table(&mut self) {
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
impl Write for ExporterScriptBuilder {
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
