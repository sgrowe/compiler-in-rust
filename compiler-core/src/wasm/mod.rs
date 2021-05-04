pub use format::{Wasm, WasmIndentation};
pub use instruction::{WasmBlock, WasmInstr, WasmType};
use std::collections::BTreeSet;
use std::fmt::{self, Write};

mod format;
mod instruction;

#[derive(Debug, Default)]
pub struct WasmModule<'a> {
    functions: Vec<WasmFunction<'a>>,
    exports: Vec<WasmExport<'a>>,
}

impl<'a> WasmModule<'a> {
    pub fn add_function(&mut self, func: WasmFunction<'a>, exported: bool) {
        let name = func.name;

        self.functions.push(func);

        if exported {
            self.exports.push(WasmExport::Function {
                wasm_name: name,
                exported_name: name,
            })
        }
    }
}

impl<'a, Writer: Write> Wasm<Writer> for WasmModule<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmIndentation) -> fmt::Result {
        write!(w, "(module")?;

        let body_format = format.increase_indent();

        for func in &self.functions {
            func.write_text(w, body_format)?;
        }

        for export in &self.exports {
            export.write_text(w, body_format)?;
        }

        write!(w, ")")
    }
}

#[derive(Debug)]
pub struct WasmFunction<'a> {
    name: &'a str,
    params: Vec<&'a str>,
    local_variables: BTreeSet<&'a str>,
    return_type: Option<WasmType>,
    body: WasmBlock<'a>,
}

impl<'a> WasmFunction<'a> {
    pub fn new(
        name: &'a str,
        params: Vec<&'a str>,
        local_variables: BTreeSet<&'a str>,
        return_type: Option<WasmType>,
        body: WasmBlock<'a>,
    ) -> WasmFunction<'a> {
        WasmFunction {
            name,
            params,
            local_variables,
            return_type,
            body,
        }
    }

    pub fn add_local_variable(&mut self, name: &'a str) {
        self.local_variables.insert(name);
    }
}

impl<'a, Writer: Write> Wasm<Writer> for WasmFunction<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmIndentation) -> fmt::Result {
        format.new_line_with_indent(w)?;

        write!(w, "(func ${}", self.name)?;

        for param in &self.params {
            write!(w, " (param ${} i32)", param)?;
        }

        if let Some(wasm_type) = self.return_type {
            write!(w, " (result {})", wasm_type.to_wasm_text())?;
        }

        for local in &self.local_variables {
            write!(w, " (local ${} i32)", local)?;
        }

        let body_format = format.increase_indent();

        for instruction in &self.body {
            instruction.write_text(w, body_format)?;
        }

        write!(w, ")")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WasmExport<'a> {
    Function {
        wasm_name: &'a str,
        exported_name: &'a str,
    },
}

impl<'a, Writer: Write> Wasm<Writer> for WasmExport<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmIndentation) -> fmt::Result {
        use WasmExport::*;

        format.new_line_with_indent(w)?;

        match self {
            Function {
                wasm_name,
                exported_name,
            } => write!(w, "(export \"{}\" (func ${}))", exported_name, wasm_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn assert_wasm_output_matches<Input: Wasm<String>>(wasm: Input, expected: &str) {
        let mut out = String::new();

        wasm.write_text(&mut out, WasmIndentation::default())
            .unwrap();

        assert_eq!(out, expected);
    }

    fn assert_wasm_snapshot_matches<Input: Wasm<String>>(snapshot: &str, wasm: Input) {
        let mut out = String::new();

        wasm.write_text(&mut out, WasmIndentation::default())
            .unwrap();

        assert_snapshot!(snapshot, out);
    }

    #[test]
    fn formats_empty_function() {
        assert_wasm_output_matches(
            WasmFunction::new("f", vec![], BTreeSet::new(), None, vec![]),
            "(func $f)",
        );
    }

    #[test]
    fn formats_single_arg_function() {
        let func = WasmFunction::new(
            "my_func",
            vec!["arg_1", "arg_2"],
            BTreeSet::new(),
            Some(WasmType::I32),
            vec![],
        );

        assert_wasm_output_matches(
            func,
            "(func $my_func (param $arg_1 i32) (param $arg_2 i32) (result i32))",
        );
    }

    #[test]
    fn formats_module_with_functions() {
        use WasmInstr::*;
        use WasmType::*;

        let mut module = WasmModule::default();

        module.add_function(
            WasmFunction::new(
                "get_magic_number",
                vec![],
                BTreeSet::new(),
                Some(I32),
                vec![ConstI32(10), ConstI32(5), AddI32],
            ),
            false,
        );

        module.add_function(
            WasmFunction::new(
                "add",
                vec!["arg_1", "arg_2"],
                BTreeSet::new(),
                Some(I32),
                vec![GetLocal("arg_1"), GetLocal("arg_2"), AddI32],
            ),
            true,
        );

        assert_wasm_snapshot_matches("module with two simple functions", module);
    }

    #[test]
    fn formats_simple_export() {
        assert_wasm_output_matches(
            WasmExport::Function {
                exported_name: "add",
                wasm_name: "add_em",
            },
            "(export \"add\" (func $add_em))",
        );
    }
}
