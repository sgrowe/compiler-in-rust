use self::wasm::*;

pub mod ast;
pub mod binding_power;
pub mod code_gen;
pub mod keywords;
pub mod operators;
pub mod parser;
pub mod tokeniser;
pub mod tokens;
pub mod wasm;

pub fn compile<'a>(source: &'a str) -> Result<String, CompileError<'a>> {
    let mut out = Vec::new();

    let ast = self::parser::parse(&source)?;

    let wasm = self::code_gen::ast_to_wasm(&ast)?;

    wasm.write_text(&mut out, WasmFormat::default())?;

    Ok(std::str::from_utf8(&out).unwrap().to_owned())
}

pub enum CompileError<'a> {
    ParseError(parser::ParseError<'a>),
    CodeGenError(code_gen::CodeGenError),
    IOError(std::io::Error),
}

impl<'a> From<parser::ParseError<'a>> for CompileError<'a> {
    fn from(error: parser::ParseError<'a>) -> Self {
        CompileError::ParseError(error)
    }
}

impl<'a> From<code_gen::CodeGenError> for CompileError<'a> {
    fn from(error: code_gen::CodeGenError) -> Self {
        CompileError::CodeGenError(error)
    }
}

impl<'a> From<std::io::Error> for CompileError<'a> {
    fn from(error: std::io::Error) -> Self {
        CompileError::IOError(error)
    }
}
