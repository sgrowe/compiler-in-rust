use super::operators::*;
use super::tokens::*;

#[derive(Debug, Default, Clone)]
pub struct Ast<'a> {
    pub statements: Vec<TopLevelStatement<'a>>,
}

impl<'a> Ast<'a> {
    pub fn append_statement(&mut self, statement: TopLevelStatement<'a>) {
        self.statements.push(statement)
    }
}

#[derive(Debug, Clone)]
pub enum Declaration<'a> {
    Assignment {
        name: &'a str,
        expr: Expression<'a>,
    },
    FunctionDecl {
        name: &'a str,
        arguments: FunctionArgsList<'a>,
        body: Vec<FuncBodyStatement<'a>>,
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

#[derive(Debug, Clone)]
pub enum TopLevelStatement<'a> {
    Declaration {
        decl: Declaration<'a>,
        exported: bool,
    },
}

#[derive(Debug, Clone)]
pub enum FuncBodyStatement<'a> {
    Declaration(Declaration<'a>),
    BareExpression(Expression<'a>),
}

#[derive(Debug, Clone)]
pub struct FunctionArgsList<'a> {
    pub args: Vec<FunctionArg<'a>>,
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionArg<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Variable(&'a str),
    Constant(Constant<'a>),
    FunctionCall {
        name: &'a str,
        args: Vec<Expression<'a>>,
    },
    BinaryOp {
        operator: BinaryOperator,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
    },
    Negation(Box<Expression<'a>>),
}
