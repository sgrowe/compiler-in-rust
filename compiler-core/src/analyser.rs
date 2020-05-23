use crate::{
    ast::{Declaration, TopLevelStatement},
    parser::{ParseError, Parser},
};
use std::collections::HashMap;

type Result<'a, X> = std::result::Result<X, AnalyserError<'a>>;

struct Symbol<'a> {
    decl: Declaration<'a>,
    exported: bool,
    usage_count: u64,
}

fn analyse(parser: Parser) -> Result<()> {
    let mut symbols = HashMap::new();

    for statement in parser {
        match statement? {
            TopLevelStatement::Declaration { decl, exported } => {
                let name = decl.name();
                let symbol = Symbol {
                    decl,
                    exported,
                    usage_count: 0,
                };

                let already_declared = symbols.insert(name, symbol).is_some();

                if already_declared {
                    return Err(AnalyserError::DuplicateVariable(name));
                };
            }
        }
    }

    let main = symbols.get("main").ok_or(AnalyserError::NoMain)?;

    Ok(())
}

pub enum AnalyserError<'a> {
    ParseError(ParseError<'a>),
    DuplicateVariable(&'a str),
    NoMain,
}

impl<'a> From<ParseError<'a>> for AnalyserError<'a> {
    fn from(error: ParseError<'a>) -> Self {
        AnalyserError::ParseError(error)
    }
}

// #[derive(Debug)]
// pub struct Analyser<'a> {
//     parser: Parser<'a>,
// }

// impl<'a> Analyser<'a> {
//     fn collect(&mut self, )
// }
