use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },

    Int(i64),

    Verify(Box<Expression>),

    /// `VirtualTarget` in plonky2
    VirtualWire {
        index: usize,
    },

    /// `Target` in plonky2
    Wire {
        row: usize,
        column: usize,
    },
}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Divide,
    Equal,
    Exponent,
    GreaterThanEqual,
    GreaterThan,
    LessThan,
    LessThanEqual,
    Multiply,
    Subtract,
    MultipleSubtract,
}
