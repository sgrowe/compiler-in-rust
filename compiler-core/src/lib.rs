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

pub fn compile(source: &str) -> String {
    let mut out = Vec::new();

    let ast = self::parser::parse(&source).unwrap();

    let wasm = self::code_gen::ast_to_wasm(&ast);

    wasm.write_text(&mut out, WasmFormat::default()).unwrap();

    std::str::from_utf8(&out).unwrap().to_owned()
}
