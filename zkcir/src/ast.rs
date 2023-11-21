use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },

    Wire {
        row: usize,
        column: usize,
    },
}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Multiply,
    Subtract,
}
