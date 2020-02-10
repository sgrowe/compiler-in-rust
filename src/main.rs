extern crate clap;

use self::code_gen::*;
use self::wasm::*;
use clap::{App, Arg};
use std::fs::{create_dir_all, File};
use std::io::BufWriter;

mod ast;
mod binding_power;
mod code_gen;
mod keywords;
mod operators;
mod parser;
mod tokeniser;
mod tokens;
mod wasm;

fn main() -> std::io::Result<()> {
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

    let ast = self::parser::parse(&source).unwrap();

    let wasm = ast_to_wasm(&ast);

    create_dir_all("dist")?;

    let mut output_file = BufWriter::new(File::create("dist/out.wat")?);

    wasm.write_text(&mut output_file, WasmFormat::default())?;

    Ok(())
}
