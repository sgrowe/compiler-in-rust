use super::keywords::Keyword;

#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    RightArrow,
    Equals,
    Pipe,
    Comma,
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
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
}

#[derive(Debug, Copy, Clone)]
pub enum Constant<'a> {
    Str(&'a str),
    Float(f64),
    Int(i64),
}
