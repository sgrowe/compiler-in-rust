pub trait BindingPower {
    fn binding_power(&self) -> u32;
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
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
