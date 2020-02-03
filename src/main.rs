extern crate clap;

use clap::{App, Arg};

mod tokeniser;

fn main() {
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

    println!("Hello, {}", file);
}
