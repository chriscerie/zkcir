use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },

    /// `Target` in plonky2
    Wire { row: usize, column: usize },

    /// `VirtualTarget` in plonky2
    VirtualWire(usize),
}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Multiply,
    Subtract,
}
