use super::ast::*;
use super::operators::*;
use super::tokens::*;
use super::wasm::*;
use std::collections::BTreeSet;

pub fn ast_to_wasm<'a>(ast: &Ast<'a>) -> Result<WasmModule<'a>, CodeGenError> {
    use self::Declaration::*;
    use TopLevelStatement::*;

    let mut module = WasmModule::default();

    for statement in &ast.statements {
        match statement {
            Declaration { decl, exported } => match decl {
                FunctionDecl {
                    name,
                    arguments,
                    body,
                } => {
                    let wasm_args = arguments.args.iter().map(|arg| arg.name).collect();

                    let mut wasm_body = Vec::with_capacity(body.len());

                    let mut locals = BTreeSet::new();

                    for statement in body {
                        compile_func_body_statement(statement, &mut wasm_body, &mut locals)?;
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

fn compile_code_block<'a>(
    block: &CodeBlock<'a>,
) -> Result<(Vec<WasmInstr<'a>>, BTreeSet<&'a str>), CodeGenError> {
    let mut instr = Vec::with_capacity(block.len()); // generally at least one instruction per statement

    let mut locals = BTreeSet::new();

    for s in block {
        compile_func_body_statement(s, &mut instr, &mut locals)?;
    }

    Ok((instr, locals))
}

fn compile_func_body_statement<'a>(
    statement: &CodeBlockStatement<'a>,
    mut instructions: &mut Vec<WasmInstr<'a>>,
    locals: &mut BTreeSet<&'a str>,
) -> Result<(), CodeGenError> {
    match statement {
        CodeBlockStatement::BareExpression(expr) => compile_expression(expr, &mut instructions)?,
        CodeBlockStatement::Declaration(Declaration::Assignment { name, expr }) => {
            compile_expression(expr, &mut instructions)?;
            locals.insert(name);
            instructions.push(WasmInstr::SetLocal(name));
        }
        CodeBlockStatement::Declaration(Declaration::FunctionDecl { .. }) => {
            return Err(CodeGenError::ClosuresNotSupportedYet)
        }
        CodeBlockStatement::IfStatement { cases, else_case } => {
            let mut fallback = match else_case {
                Some(block) => {
                    let (instr, else_locals) = compile_code_block(&block)?;

                    locals.extend(else_locals);

                    Some(instr)
                }
                None => None,
            };

            for IfStatementCase { condition, block } in cases.iter().rev() {
                let mut wasm_cond = Vec::new();

                compile_expression(&condition, &mut wasm_cond)?;

                let (then, then_locals) = compile_code_block(&block)?;

                locals.extend(then_locals);

                let instr = WasmInstr::If {
                    result_type: WasmType::I32,
                    condition: wasm_cond,
                    then,
                    else_: fallback,
                };

                fallback = Some(vec![instr])
            }

            if let Some(last_if) = fallback {
                instructions.extend(last_if);
            }
        }
    };

    Ok(())
}

fn compile_expression<'a>(
    expr: &Expression<'a>,
    mut instr: &mut Vec<WasmInstr<'a>>,
) -> Result<(), CodeGenError> {
    use self::Constant::*;
    use Expression::*;

    match expr {
        &Constant(Int(int)) => {
            instr.push(WasmInstr::ConstI32(int as i32));
        }
        &Constant(Float(float)) => {
            instr.push(WasmInstr::ConstF32(float as f32));
        }
        Constant(Str(_)) => return Err(CodeGenError::StringsNotSupportedYet),
        Variable(name) => {
            instr.push(WasmInstr::GetLocal(name));
        }
        Negation(expr) => {
            compile_expression(expr, &mut instr)?;

            instr.reserve(2);
            instr.push(WasmInstr::ConstI32(-1));
            instr.push(WasmInstr::MultiplyI32);
        }
        BinaryOp {
            operator,
            left,
            right,
        } => {
            instr.reserve(3);

            compile_expression(left, &mut instr)?;
            compile_expression(right, &mut instr)?;

            instr.push(binary_op_to_wasm_instruction(*operator));
        }
        FunctionCall { name, args } => {
            instr.reserve(args.len() + 1);

            for expr in args {
                compile_expression(expr, &mut instr)?;
            }

            instr.push(WasmInstr::Call(name));
        }
    };

    Ok(())
}

fn binary_op_to_wasm_instruction<'a>(op: BinaryOperator) -> WasmInstr<'a> {
    use BinaryOperator::*;
    use WasmInstr::*;

    match op {
        Plus => AddI32,
        Minus => MinusI32,
        Multiply => MultiplyI32,
        Divide => SignedDivideI32,
        DoubleEquals => EqualI32,
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
