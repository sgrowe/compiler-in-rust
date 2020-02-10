use wasm_bindgen::prelude::*;

use self::wasm::*;

mod ast;
mod binding_power;
mod code_gen;
mod keywords;
mod operators;
mod parser;
mod tokeniser;
mod tokens;
mod wasm;

#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    let mut out = Vec::new();

    let ast = self::parser::parse(&source).unwrap();

    let wasm = self::code_gen::ast_to_wasm(&ast);

    wasm.write_text(&mut out, WasmFormat::default()).unwrap();

    std::str::from_utf8(&out).unwrap().to_owned()
}
