use super::tokens::*;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub statements: Vec<TopLevelStatement<'a>>,
}

#[derive(Debug, Clone)]
pub enum TopLevelStatement<'a> {
    Assignment { name: &'a str, expr: Expression<'a> },
    BareExpression(Expression<'a>),
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
