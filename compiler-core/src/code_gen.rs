use super::ast::*;
use super::operators::*;
use super::tokens::*;
use super::wasm::*;
use std::collections::HashSet;

pub fn ast_to_wasm<'a>(ast: &Ast<'a>) -> Result<WasmModule<'a>, CodeGenError> {
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

                    let mut wasm_body = vec![];

                    let mut locals = HashSet::new();

                    for statement in body {
                        let (mut instructions, locals_vars) =
                            compile_func_body_statement(statement)?;

                        locals_vars.iter().for_each(|&name| {
                            locals.insert(name);
                        });

                        wasm_body.append(&mut instructions);
                    }

                    module.add_function(
                        WasmFunction::new(name, wasm_args, locals, Some(WasmType::I32), wasm_body),
                        *exported,
                    )
                }
                Assignment { name: _, expr: _ } => {
                    return Err(CodeGenError::TopLevelAssignmentNotYetSupported)
                }
            },
        }
    }

    Ok(module)
}

#[derive(Debug, Copy, Clone)]
pub enum CodeGenError {
    TopLevelAssignmentNotYetSupported,
    ClosuresNotSupportedYet,
    StringsNotSupportedYet,
}

fn compile_func_body_statement<'a>(
    statement: &FuncBodyStatement<'a>,
) -> Result<(Vec<WasmInstruction<'a>>, Vec<&'a str>), CodeGenError> {
    let mut locals = vec![];

    let instructions = match statement {
        FuncBodyStatement::BareExpression(expr) => compile_expression(expr)?,
        FuncBodyStatement::Declaration(Declaration::Assignment { name, expr }) => {
            let mut instr = compile_expression(expr)?;
            locals.push(*name);
            instr.push(WasmInstruction::SetLocal(name));
            instr
        }
        FuncBodyStatement::Declaration(Declaration::FunctionDecl {
            name: _,
            arguments: _,
            body: _,
        }) => return Err(CodeGenError::ClosuresNotSupportedYet),
    };

    Ok((instructions, locals))
}

fn compile_expression<'a>(expr: &Expression<'a>) -> Result<Vec<WasmInstruction<'a>>, CodeGenError> {
    use self::Constant::*;
    use Expression::*;

    let instructions = match expr {
        Constant(Int(int)) => vec![WasmInstruction::ConstI32(*int as i32)],
        Constant(Float(float)) => vec![WasmInstruction::ConstF32(*float as f32)],
        Constant(Str(_)) => return Err(CodeGenError::StringsNotSupportedYet),
        Variable(name) => vec![WasmInstruction::GetLocal(name)],
        BinaryOp {
            operator,
            left,
            right,
        } => {
            let mut instr = vec![];

            instr.append(&mut compile_expression(left)?);
            instr.append(&mut compile_expression(right)?);
            instr.push(binary_op_to_wasm_instruction(*operator));

            instr
        }
        FunctionCall { name, args } => {
            let mut instr = Vec::with_capacity(args.len() + 1);

            for expr in args {
                instr.append(&mut compile_expression(expr)?);
            }

            instr.push(WasmInstruction::Call(name));

            instr
        }
    };

    Ok(instructions)
}

fn binary_op_to_wasm_instruction<'a>(op: BinaryOperator) -> WasmInstruction<'a> {
    use BinaryOperator::*;
    use WasmInstruction::*;

    match op {
        Plus => AddI32,
        Minus => MinusI32,
        Multiply => MultiplyI32,
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
