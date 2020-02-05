use super::operators::*;
use super::tokens::*;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub statements: Vec<Statement<'a>>,
}

impl<'a> Ast<'a> {
    pub fn append_statement(&mut self, statement: Statement<'a>) {
        self.statements.push(statement)
    }
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Assignment {
        name: &'a str,
        expr: Expression<'a>,
    },
    BareExpression(Expression<'a>),
    FunctionDecl {
        name: &'a str,
        arguments: FunctionArgsList<'a>,
        body: Vec<Statement<'a>>,
    },
}

#[derive(Debug, Clone)]
pub struct FunctionArgsList<'a> {
    pub args: Vec<FunctionArg<'a>>,
}

#[derive(Debug, Clone)]
pub struct FunctionArg<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Variable(&'a str),
    Constant(Constant<'a>),
    BinaryOp {
        operator: BinaryOperator,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
    },
}
