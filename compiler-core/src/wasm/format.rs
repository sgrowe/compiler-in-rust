use std::fmt::{self, Write};

pub trait Wasm<Writer: fmt::Write> {
    fn write_text(&self, writer: &mut Writer, format: WasmIndentation) -> fmt::Result;
}

#[derive(Default, Debug, Copy, Clone)]
pub struct WasmIndentation {
    indent: u32,
}

impl WasmIndentation {
    pub fn increase_indent(self) -> Self {
        WasmIndentation {
            indent: self.indent + 2,
        }
    }

    pub fn new_line_with_indent<W: Write>(self, writer: &mut W) -> fmt::Result {
        if self.indent > 0 {
            writer.write_char('\n')?;

            for _ in 0..self.indent {
                writer.write_char(' ')?;
            }
        }

        Ok(())
    }
}
