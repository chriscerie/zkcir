use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },

    /// `Target` in plonky2
    Wire {
        row: usize,
        column: usize,
    },

    /// `VirtualTarget` in plonky2
    VirtualWire {
        index: usize,
    },

    Verify {
        expr: Box<Expression>,
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
}
