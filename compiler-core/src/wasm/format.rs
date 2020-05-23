use std::fmt::{self, Write};

#[derive(Default, Debug, Copy, Clone)]
pub struct WasmTextFormatOptions {
    indent: u32,
}

impl WasmTextFormatOptions {
    pub fn increase_indent(self) -> Self {
        WasmTextFormatOptions {
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
