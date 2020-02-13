use super::ast::*;
use super::operators::*;
use super::tokens::*;
use super::wasm::*;

pub fn ast_to_wasm<'a>(ast: &Ast<'a>) -> WasmModule<'a> {
    use self::Declaration::*;
    use TopLevelStatement::*;

    let mut module = WasmModule::new();

    for statement in &ast.statements {
        match statement {
            Declaration { decl, exported } => match decl {
                FunctionDecl {
                    name,
                    arguments,
                    body,
                } => {
                    let wasm_args = arguments.args.iter().map(|arg| arg.name).collect();

                    let wasm_body = body.iter().flat_map(compile_func_body_statement).collect();

                    module.add_function(
                        WasmFunction::new(name, wasm_args, Some(WasmType::I32), wasm_body),
                        *exported,
                    )
                }
                _ => panic!(),
            },
        }
    }

    module
}

fn compile_func_body_statement<'a>(statement: &FuncBodyStatement<'a>) -> Vec<WasmInstruction<'a>> {
    use FuncBodyStatement::*;

    match statement {
        BareExpression(expr) => compile_expression(expr),
        _ => panic!(),
    }
}

fn compile_expression<'a>(expr: &Expression<'a>) -> Vec<WasmInstruction<'a>> {
    use self::Constant::*;
    use Expression::*;

    match expr {
        Constant(Int(int)) => vec![WasmInstruction::ConstI32(*int as i32)],
        Variable(name) => vec![WasmInstruction::GetLocal(name)],
        BinaryOp {
            operator,
            left,
            right,
        } => {
            let mut instr = vec![];

            instr.append(&mut compile_expression(left));
            instr.append(&mut compile_expression(right));
            instr.push(binary_op_to_wasm_instruction(*operator));

            instr
        }
        FunctionCall { name, args } => {
            let mut instr = Vec::with_capacity(args.len() + 1);

            args.iter().for_each(|expr| {
                instr.append(&mut compile_expression(expr));
            });

            instr.push(WasmInstruction::Call(name));

            instr
        }
        _ => panic!(),
    }
}

fn binary_op_to_wasm_instruction<'a>(op: BinaryOperator) -> WasmInstruction<'a> {
    use BinaryOperator::*;
    use WasmInstruction::*;

    match op {
        Plus => AddI32,
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::parser::parse;
    use super::*;
    use insta::assert_debug_snapshot;
    use std::fs;
    use test_case::test_case;

    #[test_case("src/fixtures/example_program.lang"; "example program")]
    fn fixtures(fixture_file_name: &str) -> std::io::Result<()> {
        let contents = fs::read_to_string(fixture_file_name)?;

        let ast = parse(&contents).unwrap();

        let wasm = ast_to_wasm(&ast);

        assert_debug_snapshot!(wasm);

        Ok(())
    }
}
