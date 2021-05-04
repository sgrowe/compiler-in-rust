use self::wasm::*;

pub mod analyser;
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

    wasm.write_text(&mut out, WasmIndentation::default())?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use test_case::test_case;
    use wasmtime::*;

    #[test_case("fibonacci", 0, 0)]
    #[test_case("fibonacci", 1, 1)]
    #[test_case("fibonacci", 10, 55)]
    #[test_case("fibonacci", 12, 144)]
    fn program<Args>(name: &str, args: Args, expected: i32)
    where
        Args: WasmParams,
    {
        let code = fs::read_to_string(&format!("src/fixtures/{}.lang", name)).unwrap();

        let wasm = compile(&code).unwrap();

        let engine = Engine::default();

        let store = Store::new(&engine);

        let module = Module::new(&engine, &wasm).unwrap();

        let instance = Instance::new(&store, &module, &[]).unwrap();

        let main = instance
            .get_func("main")
            .expect("`main` was not an exported function");

        let answer = main.typed::<Args, i32>().unwrap();

        let result = answer.call(args).unwrap();

        assert_eq!(result, expected);
    }
}
