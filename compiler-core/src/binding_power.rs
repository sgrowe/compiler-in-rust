use super::operators::*;
use super::tokens::*;

pub trait BindingPower {
    fn binding_power(self) -> u32;
}

impl BindingPower for BinaryOperator {
    fn binding_power(self) -> u32 {
        use BinaryOperator::*;

        match self {
            Plus | Minus => 50,
            Multiply | Divide => 60,
        }
    }
}

impl<'a> BindingPower for Token<'a> {
    fn binding_power(self) -> u32 {
        match self {
            Token::BinOp(op) => op.binding_power(),
            _ => 0,
        }
    }
}
