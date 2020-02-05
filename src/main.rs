extern crate clap;

use clap::{App, Arg};

mod ast;
mod keywords;
mod operators;
mod parser;
mod tokeniser;
mod tokens;

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

    let ast = self::parser::parse(&source);

    println!("Hello, {:#?}", ast);

    Ok(())
}
