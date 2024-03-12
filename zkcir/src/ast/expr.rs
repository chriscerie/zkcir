extern crate alloc;

use alloc::{boxed::Box, format, string::ToString};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::node::Node;

use super::{Ident, Op};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
    },
    Ident(Ident),
    Value(Value),
    Unary {
        op: Op,
        expr: Box<Expression>,
    },
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
            Expression::Ident(ident) => ident.visit_values(f),
            Expression::Unary { expr, .. } => expr.visit_values(f),
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
            Expression::Ident(ident) => ident.visit_virtual_wires(f),
            Expression::Value(_) => {}
            Expression::Unary { expr, .. } => expr.visit_virtual_wires(f),
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
            Expression::Ident(ident) => ident.visit_wires(f),
            Expression::Value(_) => {}
            Expression::Unary { expr, .. } => expr.visit_wires(f),
        }
    }

    fn visit_expressions_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                *lhs = Box::new(f(lhs));
                *rhs = Box::new(f(rhs));
            }
            Expression::Ident(ident) => ident.visit_expressions_mut(f),
            Expression::Value(value) => {
                value.visit_expressions_mut(f);
            }
            Expression::Unary { expr, .. } => {
                *expr = Box::new(f(expr));
            }
        }
    }

    fn visit_idents_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_idents_mut(f);
                rhs.visit_idents_mut(f);
            }
            Expression::Ident(ident) => *ident = f(ident),
            Expression::Value(value) => {
                value.visit_idents_mut(f);
            }
            Expression::Unary { expr, .. } => {
                expr.visit_idents_mut(f);
            }
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
            Expression::Ident(ident) => ident.to_code_ir(),
            Expression::Unary { op, expr } => {
                format!("{}{}", op.to_code_ir(), expr.to_code_ir())
            }
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

    fn visit_expressions_mut<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
    }

    fn visit_idents_mut<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
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
    pub wiretype: Wiretype,
}

impl VirtualWire {
    #[must_use]
    pub fn new_public(index: usize) -> Self {
        Self {
            index,
            value: None,
            wiretype: Wiretype::Public,
        }
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

    fn visit_expressions_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
        if let Some(value) = &mut self.value {
            value.visit_expressions_mut(f);
        }
    }

    fn visit_idents_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
    {
        if let Some(value) = &mut self.value {
            value.visit_idents_mut(f);
        }
    }

    fn to_code_ir(&self) -> alloc::string::String {
        if let Some(value) = &self.value {
            format!(
                "virtual_wire::{}(index: {}, value: {})",
                self.wiretype,
                self.index,
                value.to_code_ir()
            )
        } else {
            format!("virtual_wire::{}(index: {})", self.wiretype, self.index)
        }
    }
}

impl From<VirtualWire> for Expression {
    fn from(val: VirtualWire) -> Self {
        Expression::Ident(Ident::VirtualWire(val))
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug, Display)]
pub enum Wiretype {
    /// * In halo2, instance columns
    /// * In plonky2, wires registered as public input
    #[display(fmt = "public")]
    Public,

    /// * In halo2, advice columns
    /// * In plonky2, wires not registered as public input
    #[display(fmt = "private")]
    Private,

    /// * In halo2, fixed columns
    #[display(fmt = "const")]
    Constant,
}

/// `Target` in plonky2
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Wire {
    pub row: usize,
    pub column: usize,
    pub value: Option<Value>,
    pub wiretype: Wiretype,
}

impl Wire {
    #[must_use]
    pub fn new_constant(row: usize, column: usize) -> Wire {
        Self {
            row,
            column,
            value: None,
            wiretype: Wiretype::Constant,
        }
    }

    #[must_use]
    pub fn new_private(row: usize, column: usize) -> Wire {
        Self {
            row,
            column,
            value: None,
            wiretype: Wiretype::Private,
        }
    }

    #[must_use]
    pub fn new_public(row: usize, column: usize) -> Wire {
        Self {
            row,
            column,
            value: None,
            wiretype: Wiretype::Public,
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

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        if let Some(value) = &mut self.value {
            value.visit_virtual_wires(f);
        }
    }

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        f(self);
    }

    fn visit_expressions_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
        if let Some(value) = &mut self.value {
            value.visit_expressions_mut(f);
        }
    }

    fn visit_idents_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
    {
        if let Some(value) = &mut self.value {
            value.visit_idents_mut(f);
        }
    }

    fn to_code_ir(&self) -> alloc::string::String {
        if let Some(value) = &self.value {
            format!(
                "wire::{}(row: {}, column: {}, value: {})",
                self.wiretype,
                self.row,
                self.column,
                value.to_code_ir()
            )
        } else {
            format!(
                "wire::{}(row: {}, column: {})",
                self.wiretype, self.row, self.column
            )
        }
    }
}

impl From<Wire> for Expression {
    fn from(val: Wire) -> Self {
        Expression::Ident(Ident::Wire(val))
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
                            wiretype: Wiretype::Private,
                        }
                        .into(),
                    ),
                    binop: BinOp::Add,
                    rhs: Box::new(VirtualWire::new_public(3).into()),
                }),
                binop: BinOp::Multiply,
                rhs: Box::new(Wire::new_private(5, 6).into()),
            }
            .to_code_ir(),
        );
    }
}
