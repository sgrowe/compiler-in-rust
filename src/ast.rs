use super::operators::*;
use super::tokens::*;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    statements: Vec<TopLevelStatement<'a>>,
}

impl<'a> Ast<'a> {
    pub fn new() -> Ast<'a> {
        Ast {
            statements: Vec::new(),
        }
    }

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