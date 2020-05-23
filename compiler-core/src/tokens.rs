use super::keywords::Keyword;
use super::operators::BinaryOperator;

#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    Equals,
    Pipe,
    Comma,
    Colon,
    FatRightArrow,
    OpenParen,
    CloseParen,
    IndentIncr,
    IndentDecr,
    Name(&'a str),
    Keyword(Keyword),
    BinOp(BinaryOperator),
    Constant(Constant<'a>),
}

#[derive(Debug, Copy, Clone)]
pub enum Constant<'a> {
    Str(&'a str),
    Float(f64),
    Int(i64),
}
