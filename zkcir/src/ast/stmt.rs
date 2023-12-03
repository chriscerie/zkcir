extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
};
use serde::{Deserialize, Serialize};

use crate::node::Node;

use super::{
    expr::{Expression, Value, VirtualWire, Wire},
    ident::Ident,
};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum Stmt {
    Verify(Expression),

    /// Local assignment like `let x = y;`
    Local(Ident, Expression),
}

impl Node for Stmt {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        match self {
            Stmt::Verify(expr) => {
                expr.visit_values(f);
            }
            Stmt::Local(_ident, stmt) => stmt.visit_values(f),
        }
    }

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        match self {
            Stmt::Verify(expr) => {
                expr.visit_virtual_wires(f);
            }
            Stmt::Local(_ident, stmt) => stmt.visit_virtual_wires(f),
        }
    }

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        match self {
            Stmt::Verify(expr) => {
                expr.visit_wires(f);
            }
            Stmt::Local(_ident, stmt) => stmt.visit_wires(f),
        }
    }

    fn to_code_ir(&self) -> String {
        match self {
            Stmt::Verify(stmt) => format!("verify!({});", stmt.to_code_ir()),
            Stmt::Local(ident, stmt) => {
                format!("let {} = {};", ident.to_code_ir(), stmt.to_code_ir())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::{ast::expr::BinOp, test_util::test_code_ir};

    use super::*;

    #[test]
    fn test_valid_stmt_source() {
        test_code_ir(
            "valid_stmt_source",
            &Stmt::Local(
                Ident::Wire(Wire::new(3, 2)),
                Expression::BinaryOperator {
                    lhs: Box::new(Expression::BinaryOperator {
                        lhs: Box::new(Wire::new(1, 2).into()),
                        binop: BinOp::Add,
                        rhs: Box::new(VirtualWire::new(3).into()),
                    }),
                    binop: BinOp::Multiply,
                    rhs: Box::new(Wire::new(5, 6).into()),
                },
            )
            .to_code_ir(),
        );
    }
}
