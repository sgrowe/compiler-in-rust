use super::format::{Wasm, WasmIndentation};
use std::fmt::{self, Write};

#[derive(Debug, Copy, Clone)]
pub enum WasmInstruction<'a> {
    GetLocal(&'a str),
    SetLocal(&'a str),
    ConstI64(i64),
    ConstI32(i32),
    ConstF32(f32),
    AddI64,
    AddI32,
    MinusI32,
    MultiplyI32,
    SignedDivideI32,
    Call(&'a str),
}

impl<'a, Writer: Write> Wasm<Writer> for WasmInstruction<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmIndentation) -> fmt::Result {
        use WasmInstruction::*;

        format.new_line_with_indent(w)?;

        match self {
            GetLocal(name) => write!(w, "local.get ${}", name),
            SetLocal(name) => write!(w, "local.set ${}", name),
            ConstI64(value) => write!(w, "i64.const {}", value),
            ConstI32(value) => write!(w, "i32.const {}", value),
            ConstF32(value) => write!(w, "f32.const {}", value),
            AddI64 => write!(w, "i64.add"),
            AddI32 => write!(w, "i32.add"),
            MinusI32 => write!(w, "i32.sub"),
            MultiplyI32 => write!(w, "i32.mul"),
            SignedDivideI32 => write!(w, "i32.div_s"),
            Call(name) => write!(w, "call ${}", name),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WasmType {
    I32,
    I64,
}

impl WasmType {
    pub fn to_wasm_text(self) -> &'static str {
        match self {
            WasmType::I64 => "i64",
            WasmType::I32 => "i32",
        }
    }
}
