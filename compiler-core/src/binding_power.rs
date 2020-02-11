use super::operators::*;
use super::tokens::*;

pub trait BindingPower {
    fn binding_power(&self) -> u32;
}

impl BindingPower for BinaryOperator {
    fn binding_power(&self) -> u32 {
        use BinaryOperator::*;

        match self {
            Plus => 50,
            Minus => 50,
            Multiply => 60,
        }
    }
}

impl<'a> BindingPower for Token<'a> {
    fn binding_power(&self) -> u32 {
        use Token::*;

        match self {
            BinOp(op) => op.binding_power(),
            _ => 0,
        }
    }
}
