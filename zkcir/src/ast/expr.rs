extern crate alloc;

use alloc::{boxed::Box, format, string::ToString};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::node::Node;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },
    Value(Value),
    VirtualWire(VirtualWire),
    Wire(Wire),
}

impl Node for Expression {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_values(f);
                rhs.visit_values(f);
            }
            Expression::Value(value) => value.visit_values(f),
            Expression::VirtualWire(virtual_wire) => {
                virtual_wire.visit_values(f);
            }
            Expression::Wire(_) => {}
        }
    }

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_virtual_wires(f);
                rhs.visit_virtual_wires(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                virtual_wire.visit_virtual_wires(f);
            }
            Expression::Wire(wire) => {
                wire.visit_virtual_wires(f);
            }
            Expression::Value(_) => {}
        }
    }

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_wires(f);
                rhs.visit_wires(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                virtual_wire.visit_wires(f);
            }
            Expression::Wire(wire) => {
                wire.visit_wires(f);
            }
            Expression::Value(_) => {}
        }
    }

    fn to_code_ir(&self) -> alloc::string::String {
        match self {
            Expression::BinaryOperator { lhs, binop, rhs } => {
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

                format!("{lhs_str} {binop} {rhs_str}")
            }
            Expression::Value(value) => value.to_code_ir(),
            Expression::VirtualWire(virtual_wire) => virtual_wire.to_code_ir(),
            Expression::Wire(wire) => wire.to_code_ir(),
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Display)]
pub enum Value {
    #[display(fmt = "{_0}u64")]
    U64(u64),

    #[display(fmt = "(random!() -> {_0}u64)")]
    RandomU64(u64),

    /// Enables generating deterministic IRs even when using random values. Useful for snapshot tests
    #[display(fmt = "random!()")]
    Random,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Display)]
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

impl Node for Value {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        f(self);
    }

    fn visit_virtual_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
    }

    fn visit_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
    }

    fn to_code_ir(&self) -> alloc::string::String {
        self.to_string()
    }
}

/// `VirtualTarget` in plonky2
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
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

impl Node for VirtualWire {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        if let Some(value) = &mut self.value {
            value.visit_values(f);
        }
    }

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        f(self);
    }

    fn visit_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
    }

    fn to_code_ir(&self) -> alloc::string::String {
        if let Some(value) = &self.value {
            format!(
                "virtual_wire!(index: {}, value: {})",
                self.index,
                value.to_code_ir()
            )
        } else {
            format!("virtual_wire!(index: {})", self.index)
        }
    }
}

impl From<VirtualWire> for Expression {
    fn from(val: VirtualWire) -> Self {
        Expression::VirtualWire(val)
    }
}

/// `Target` in plonky2
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
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

impl Node for Wire {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        if let Some(value) = &mut self.value {
            value.visit_values(f);
        }
    }

    fn visit_virtual_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
    }

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        f(self);
    }

    fn to_code_ir(&self) -> alloc::string::String {
        if let Some(value) = &self.value {
            format!(
                "wire!(row: {}, column: {}, value: {})",
                self.row,
                self.column,
                value.to_code_ir()
            )
        } else {
            format!("wire!(row: {}, column: {})", self.row, self.column)
        }
    }
}

impl From<Wire> for Expression {
    fn from(val: Wire) -> Self {
        Expression::Wire(val)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::{ast::expr::BinOp, test_util::test_code_ir};

    use super::*;

    #[test]
    fn test_expr_valid_source() {
        test_code_ir(
            "expr_valid_source",
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
                }),
                binop: BinOp::Multiply,
                rhs: Box::new(Wire::new(5, 6).into()),
            }
            .to_code_ir(),
        );
    }
}
