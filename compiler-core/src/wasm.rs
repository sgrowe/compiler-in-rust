use std::io;
use std::io::Write;

pub trait Wasm<Writer: io::Write> {
    fn write_text(&self, writer: &mut Writer, format: WasmFormat) -> io::Result<()>;
}

#[derive(Default, Debug, Copy, Clone)]
pub struct WasmFormat {
    indent: u32,
}

impl WasmFormat {
    fn increase_indent(self) -> Self {
        WasmFormat {
            indent: self.indent + 2,
        }
    }

    fn new_line_with_indent<W: Write>(self, writer: &mut W) -> io::Result<()> {
        if self.indent > 0 {
            writer.write_all(b"\n")?;

            for _ in 0..self.indent {
                writer.write_all(b" ")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WasmModule<'a> {
    functions: Vec<WasmFunction<'a>>,
    exports: Vec<WasmExport<'a>>,
}

impl<'a> WasmModule<'a> {
    pub fn new() -> Self {
        WasmModule {
            functions: Vec::new(),
            exports: Vec::new(),
        }
    }

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
    fn write_text(&self, writer: &mut Writer, format: WasmFormat) -> io::Result<()> {
        write!(writer, "(module")?;

        let body_format = format.increase_indent();

        for func in &self.functions {
            func.write_text(writer, body_format)?;
        }

        for export in &self.exports {
            export.write_text(writer, body_format)?;
        }

        write!(writer, ")")
    }
}

#[derive(Debug, Clone)]
pub struct WasmFunction<'a> {
    name: &'a str,
    params: Vec<&'a str>,
    return_type: Option<WasmType>,
    body: Vec<WasmInstruction<'a>>,
}

impl<'a> WasmFunction<'a> {
    pub fn new(
        name: &'a str,
        params: Vec<&'a str>,
        return_type: Option<WasmType>,
        body: Vec<WasmInstruction<'a>>,
    ) -> WasmFunction<'a> {
        WasmFunction {
            name,
            params,
            return_type,
            body,
        }
    }
}

impl<'a, Writer: Write> Wasm<Writer> for WasmFunction<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmFormat) -> io::Result<()> {
        format.new_line_with_indent(w)?;

        write!(w, "(func ${}", self.name)?;

        for param in &self.params {
            write!(w, " (param ${} i32)", param)?;
        }

        if let Some(wasm_type) = self.return_type {
            write!(w, " (result {})", wasm_type.to_wasm_text())?;
        }

        let body_format = format.increase_indent();

        for instruction in &self.body {
            instruction.write_text(w, body_format)?;
        }

        write!(w, ")")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WasmInstruction<'a> {
    GetLocal(&'a str),
    ConstI64(i64),
    ConstI32(i32),
    AddI64,
    AddI32,
}

impl<'a, Writer: Write> Wasm<Writer> for WasmInstruction<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmFormat) -> io::Result<()> {
        use WasmInstruction::*;

        format.new_line_with_indent(w)?;

        match self {
            GetLocal(name) => write!(w, "local.get ${}", name),
            ConstI64(value) => write!(w, "i64.const {}", value),
            ConstI32(value) => write!(w, "i32.const {}", value),
            AddI64 => write!(w, "i64.add"),
            AddI32 => write!(w, "i32.add"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WasmExport<'a> {
    Function {
        wasm_name: &'a str,
        exported_name: &'a str,
    },
}
#[derive(Debug, Copy, Clone)]
pub enum WasmType {
    I32,
    I64,
}

impl WasmType {
    fn to_wasm_text(self) -> &'static str {
        match self {
            WasmType::I64 => "i64",
            WasmType::I32 => "i32",
        }
    }
}

impl<'a, Writer: Write> Wasm<Writer> for WasmExport<'a> {
    fn write_text(&self, writer: &mut Writer, format: WasmFormat) -> io::Result<()> {
        use WasmExport::*;

        format.new_line_with_indent(writer)?;

        match self {
            Function {
                wasm_name,
                exported_name,
            } => write!(
                writer,
                "(export \"{}\" (func ${}))",
                exported_name, wasm_name
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    fn assert_wasm_output_matches<Input: Wasm<Vec<u8>>>(wasm: Input, output: &str) {
        let mut out = Vec::new();

        wasm.write_text(&mut out, WasmFormat::default()).unwrap();

        assert_eq!(std::str::from_utf8(&out).unwrap(), output);
    }

    fn assert_wasm_snapshot_matches<Input: Wasm<Vec<u8>>>(snapshot: &str, wasm: Input) {
        let mut out = Vec::new();

        wasm.write_text(&mut out, WasmFormat::default()).unwrap();

        assert_snapshot!(snapshot, std::str::from_utf8(&out).unwrap());
    }

    #[test]
    fn formats_empty_module() {
        assert_wasm_output_matches(WasmModule::new(), "(module)");
    }

    #[test]
    fn formats_empty_function() {
        assert_wasm_output_matches(WasmFunction::new("f", vec![], None, vec![]), "(func $f)");
    }

    #[test]
    fn formats_single_arg_function() {
        let func = WasmFunction::new(
            "my_func",
            vec!["arg_1", "arg_2"],
            Some(WasmType::I64),
            vec![],
        );

        assert_wasm_output_matches(
            func,
            "(func $my_func (param $arg_1 i32) (param $arg_2 i32) (result i64))",
        );
    }

    #[test]
    fn formats_module_with_functions() {
        use WasmInstruction::*;
        use WasmType::*;

        let mut module = WasmModule::new();

        module.add_function(
            WasmFunction::new(
                "get_magic_number",
                vec![],
                Some(I64),
                vec![ConstI64(10), ConstI64(5), AddI64],
            ),
            false,
        );

        module.add_function(
            WasmFunction::new(
                "add",
                vec!["arg_1", "arg_2"],
                Some(I64),
                vec![GetLocal("arg_1"), GetLocal("arg_2"), AddI64],
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