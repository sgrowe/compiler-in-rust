use super::format::{Wasm, WasmIndentation};
use std::fmt::{self, Write};

pub type WasmBlock<'a> = Vec<WasmInstr<'a>>;

#[derive(Debug)]
pub enum WasmInstr<'a> {
    GetLocal(&'a str),
    SetLocal(&'a str),
    ConstI32(i32),
    ConstF32(f32),
    AddI32,
    MinusI32,
    MultiplyI32,
    SignedDivideI32,
    EqualI32,
    Call(&'a str),
    If {
        result_type: WasmType,
        condition: WasmBlock<'a>,
        then: WasmBlock<'a>,
        else_: Option<WasmBlock<'a>>,
    },
}

impl<'a, Writer: Write> Wasm<Writer> for WasmInstr<'a> {
    fn write_text(&self, w: &mut Writer, format: WasmIndentation) -> fmt::Result {
        format.new_line_with_indent(w)?;

        match self {
            WasmInstr::GetLocal(name) => write!(w, "local.get ${}", name),
            WasmInstr::SetLocal(name) => write!(w, "local.set ${}", name),
            WasmInstr::ConstI32(value) => write!(w, "i32.const {}", value),
            WasmInstr::ConstF32(value) => write!(w, "f32.const {}", value),
            WasmInstr::AddI32 => write!(w, "i32.add"),
            WasmInstr::MinusI32 => write!(w, "i32.sub"),
            WasmInstr::MultiplyI32 => write!(w, "i32.mul"),
            WasmInstr::SignedDivideI32 => write!(w, "i32.div_s"),
            WasmInstr::EqualI32 => write!(w, "i32.eq"),
            WasmInstr::Call(name) => write!(w, "call ${}", name),
            WasmInstr::If {
                result_type,
                condition,
                then,
                else_,
            } => {
                for instruction in condition {
                    instruction.write_text(w, format)?;
                }

                format.new_line_with_indent(w)?;
                write!(w, " (if (result {})", result_type.to_wasm_text())?;

                let indent_1 = format.increase_indent();

                indent_1.new_line_with_indent(w)?;
                write!(w, "(then")?;

                let body_format = indent_1.increase_indent();
                for instruction in then {
                    instruction.write_text(w, body_format)?;
                }

                indent_1.new_line_with_indent(w)?;
                write!(w, ")")?;

                if let Some(block) = else_ {
                    indent_1.new_line_with_indent(w)?;
                    write!(w, "(else")?;

                    let body_format = indent_1.increase_indent();
                    for instruction in block {
                        instruction.write_text(w, body_format)?;
                    }

                    indent_1.new_line_with_indent(w)?;
                    write!(w, ")")?;
                }

                write!(w, ")")
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WasmType {
    I32,
}

impl WasmType {
    pub fn to_wasm_text(self) -> &'static str {
        match self {
            WasmType::I32 => "i32",
        }
    }
}
