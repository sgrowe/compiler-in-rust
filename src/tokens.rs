#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    RightArrow,
    Equals,
    Pipe,
    Name(&'a str),
    Keyword(Keyword),
    BinOp(BinaryOperator),
    Constant(Constant<'a>),
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
}

#[derive(Debug, Copy, Clone)]
pub enum Constant<'a> {
    Str(&'a str),
    Float(f64),
    Int(i64),
}

#[derive(Debug, Copy, Clone)]
pub enum Keyword {
    Import,
    Export,
    Type,
    Function,
}
