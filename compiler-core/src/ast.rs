use super::operators::*;
use super::tokens::*;

#[derive(Debug, Default)]
pub struct Ast<'a> {
    pub statements: Vec<TopLevelStatement<'a>>,
}

impl<'a> Ast<'a> {
    pub fn append_statement(&mut self, statement: TopLevelStatement<'a>) {
        self.statements.push(statement)
    }
}

#[derive(Debug)]
pub enum Declaration<'a> {
    Assignment {
        name: &'a str,
        expr: Expression<'a>,
    },
    FunctionDecl {
        name: &'a str,
        arguments: FunctionArgsList<'a>,
        body: Vec<CodeBlockStatement<'a>>,
    },
}

impl<'a> Declaration<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Declaration::Assignment { name, expr: _ } => name,
            Declaration::FunctionDecl {
                name,
                arguments: _,
                body: _,
            } => name,
        }
    }
}

#[derive(Debug)]
pub enum TopLevelStatement<'a> {
    Declaration {
        decl: Declaration<'a>,
        exported: bool,
    },
}

pub type CodeBlock<'a> = Vec<CodeBlockStatement<'a>>;

#[derive(Debug)]
pub enum CodeBlockStatement<'a> {
    Declaration(Declaration<'a>),
    BareExpression(Expression<'a>),
    IfStatement {
        cases: Vec<IfStatementCase<'a>>,
        else_case: Option<Box<CodeBlock<'a>>>,
    },
}

#[derive(Debug)]
pub struct IfStatementCase<'a> {
    pub condition: Expression<'a>,
    pub block: CodeBlock<'a>,
}

#[derive(Debug)]
pub struct FunctionArgsList<'a> {
    pub args: Vec<FunctionArg<'a>>,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionArg<'a> {
    pub name: &'a str,
}

#[derive(Debug)]
pub enum Expression<'a> {
    Variable(&'a str),
    Constant(Constant<'a>),
    FunctionCall {
        name: &'a str,
        args: Vec<Expression<'a>>,
    },
    BinaryOp {
        left: Box<Expression<'a>>,
        operator: BinaryOperator,
        right: Box<Expression<'a>>,
    },
    Negation(Box<Expression<'a>>),
}
