use clap::{App, Arg};
use compiler_core::code_gen::*;
use compiler_core::parser::parse;
use compiler_core::wasm::*;
use std::fs::{self, create_dir_all};

fn main() -> anyhow::Result<()> {
    let matches = App::new("Lang")
        .version("0.1.0")
        .about("Rust version of lang")
        .arg(
            Arg::with_name("file")
                .takes_value(true)
                .required(true)
                .help("A cool file"),
        )
        .get_matches();

    let file = matches.value_of("file").unwrap();

    let source = std::fs::read_to_string(file)?;

    let ast = parse(&source).unwrap();

    let wasm = ast_to_wasm(&ast).unwrap();

    create_dir_all("dist")?;

    let mut out = String::new();

    wasm.write_text(&mut out, WasmTextFormatOptions::default())?;

    fs::write("dist/out.wat", out)?;

    Ok(())
}
