use std::io::{self, Write};

#[derive(Default, Debug, Copy, Clone)]
pub struct WasmFormat {
    indent: u32,
}

impl WasmFormat {
    pub fn increase_indent(self) -> Self {
        WasmFormat {
            indent: self.indent + 2,
        }
    }

    pub fn new_line_with_indent<W: Write>(self, writer: &mut W) -> io::Result<()> {
        if self.indent > 0 {
            writer.write_all(b"\n")?;

            for _ in 0..self.indent {
                writer.write_all(b" ")?;
            }
        }

        Ok(())
    }
}
