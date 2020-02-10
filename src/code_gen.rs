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

                    let wasm_body = body
                        .iter()
                        .flat_map(|s| compile_func_body_statement(&s))
                        .collect();

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
            instr.push(binary_op_to_wasm_instruction(operator));

            instr
        }
        _ => panic!(),
    }
}

fn binary_op_to_wasm_instruction<'a>(op: &BinaryOperator) -> WasmInstruction<'a> {
    use BinaryOperator::*;
    use WasmInstruction::*;

    match op {
        Plus => AddI32,
        _ => panic!(),
    }
}
