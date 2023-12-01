extern crate alloc;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
use derive_more::Display;
use serde::Serialize;

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
        result: Option<Box<Expression>>,
    },

    Value(Value),
    Verify(Box<Expression>),
    VirtualWire(VirtualWire),
    Wire(Wire),
}

impl Expression {
    pub fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_values(f);
                rhs.visit_values(f);

                if let Some(result) = result {
                    result.visit_values(f);
                }
            }
            Expression::Value(value) => {
                f(value);
            }
            Expression::Verify(expr) => {
                expr.visit_values(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                if let Some(value) = &mut virtual_wire.value {
                    f(value);
                }
            }
            Expression::Wire(wire) => {
                if let Some(value) = &mut wire.value {
                    f(value);
                }
            }
        }
    }

    pub fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_virtual_wires(f);
                rhs.visit_virtual_wires(f);

                if let Some(result) = result {
                    result.visit_virtual_wires(f);
                }
            }
            Expression::Verify(expr) => {
                expr.visit_virtual_wires(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                f(virtual_wire);
            }
            Expression::Wire(_) | Expression::Value(_) => {}
        }
    }

    pub fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_wires(f);
                rhs.visit_wires(f);

                if let Some(result) = result {
                    result.visit_wires(f);
                }
            }
            Expression::Verify(expr) => {
                expr.visit_wires(f);
            }
            Expression::Wire(wire) => {
                f(wire);
            }
            Expression::VirtualWire { .. } | Expression::Value(_) => {}
        }
    }

    #[must_use]
    pub fn to_code_ir(&self) -> String {
        match self {
            Expression::BinaryOperator {
                lhs,
                binop,
                rhs,
                result,
            } => {
                let lhs_str = if let Expression::BinaryOperator { .. } = **lhs {
                    format!("({})", lhs.to_code_ir())
                } else {
                    lhs.to_code_ir()
                };

                let rhs_str = if let Expression::BinaryOperator { .. } = **rhs {
                    format!("({})", rhs.to_code_ir())
                } else {
                    rhs.to_code_ir()
                };

                if let Some(result) = result {
                    format!("({lhs_str} {binop} {rhs_str}) -> {}", result.to_code_ir())
                } else {
                    format!("{lhs_str} {binop} {rhs_str}")
                }
            }
            Expression::Value(value) => value.to_string(),
            Expression::Verify(expression) => format!("verify!({})", expression.to_code_ir()),
            Expression::VirtualWire(virtual_wire) => {
                if let Some(value) = virtual_wire.value {
                    format!(
                        "virtual_wire!(index: {}, value: {})",
                        virtual_wire.index, value
                    )
                } else {
                    format!("virtual_wire!(index: {})", virtual_wire.index)
                }
            }
            Expression::Wire(wire) => {
                if let Some(value) = wire.value {
                    format!(
                        "wire!(row: {}, column: {}, value: {value})",
                        wire.row, wire.column
                    )
                } else {
                    format!("wire!(row: {}, column: {})", wire.row, wire.column)
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug, Display)]
pub enum Value {
    #[display(fmt = "{_0}u64")]
    U64(u64),

    #[display(fmt = "(random!() -> {_0}u64)")]
    RandomU64(u64),

    /// Enables generating deterministic IRs even when using random values. Useful for snapshot tests
    #[display(fmt = "random!()")]
    Random,
}

/// `VirtualTarget` in plonky2
#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
pub struct VirtualWire {
    pub index: usize,
    pub value: Option<Value>,
}

impl VirtualWire {
    #[must_use]
    pub fn new(index: usize) -> Self {
        Self { index, value: None }
    }
}

impl From<VirtualWire> for Expression {
    fn from(val: VirtualWire) -> Self {
        Expression::VirtualWire(val)
    }
}

/// `Target` in plonky2
#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
pub struct Wire {
    pub row: usize,
    pub column: usize,
    pub value: Option<Value>,
}

impl Wire {
    #[must_use]
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            value: None,
        }
    }
}

impl From<Wire> for Expression {
    fn from(val: Wire) -> Self {
        Expression::Wire(val)
    }
}

#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug, Display)]
pub enum BinOp {
    #[display(fmt = "+")]
    Add,

    #[display(fmt = "/")]
    Divide,

    #[display(fmt = "==")]
    Equal,

    #[display(fmt = "^")]
    Exponent,

    #[display(fmt = ">=")]
    GreaterThanEqual,

    #[display(fmt = ">")]
    GreaterThan,

    #[display(fmt = "<")]
    LessThan,

    #[display(fmt = "<=")]
    LessThanEqual,

    #[display(fmt = "*")]
    Multiply,

    #[display(fmt = "-")]
    Subtract,
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::{
        ast::{BinOp, VirtualWire, Wire},
        test_util::test_code_ir,
    };

    use super::*;

    #[test]
    fn test_valid_code_ir() {
        test_code_ir(
            "valid_code_ir",
            &Expression::BinaryOperator {
                lhs: Box::new(Expression::BinaryOperator {
                    lhs: Box::new(
                        Wire {
                            row: 1,
                            column: 2,
                            value: Some(Value::U64(5)),
                        }
                        .into(),
                    ),
                    binop: BinOp::Add,
                    rhs: Box::new(VirtualWire::new(3).into()),
                    result: Some(Box::new(Expression::Value(Value::U64(23)))),
                }),
                binop: BinOp::Multiply,
                rhs: Box::new(Wire::new(5, 6).into()),
                result: None,
            }
            .to_code_ir(),
        );
    }
}
