use compiler_core::code_gen::*;
use compiler_core::parser::parse;
use compiler_core::wasm::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    let ast = parse(&source).unwrap();

    let wasm = ast_to_wasm(&ast);

    let mut output = vec![];

    wasm.write_text(&mut output, WasmFormat::default()).unwrap();

    std::str::from_utf8(&output).unwrap().to_owned()
}
