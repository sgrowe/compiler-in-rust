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

pub fn compile(source: &str) -> Result<String, CompileError> {
    let mut out = String::new();

    let ast = self::parser::parse(&source)?;

    let wasm = self::code_gen::ast_to_wasm(&ast)?;

    wasm.write_text(&mut out, WasmTextFormatOptions::default())?;

    Ok(out)
}

#[derive(Debug)]
pub enum CompileError<'a> {
    ParseError(parser::ParseError<'a>),
    CodeGenError(code_gen::CodeGenError),
    FmtError(std::fmt::Error),
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

impl<'a> From<std::fmt::Error> for CompileError<'a> {
    fn from(error: std::fmt::Error) -> Self {
        CompileError::FmtError(error)
    }
}
