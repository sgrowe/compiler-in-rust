use super::operators::*;
use super::tokens::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingPower(u8);

impl BindingPower {
    pub fn negation() -> Self {
        BindingPower(100)
    }
}

pub trait ExpressionToken {
    fn binding_power(self) -> BindingPower;
}

impl ExpressionToken for BinaryOperator {
    fn binding_power(self) -> BindingPower {
        use BinaryOperator::*;

        let p = match self {
            Plus | Minus => 50,
            Multiply | Divide => 60,
            DoubleEquals => 90,
        };

        BindingPower(p)
    }
}

impl<'a> ExpressionToken for Token<'a> {
    fn binding_power(self) -> BindingPower {
        match self {
            Token::BinOp(op) => op.binding_power(),
            _ => BindingPower::default(),
        }
    }
}
