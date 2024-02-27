extern crate alloc;

use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};

use crate::node::Node;

use super::{Expression, Ident, Value, VirtualWire, Wire};

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum Op {
    Sub,
}

impl Node for Op {
    fn visit_values<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        {}
    }

    fn visit_virtual_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        {}
    }

    fn visit_wires<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        {}
    }

    fn visit_expressions_mut<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
        {}
    }

    fn visit_idents_mut<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
    {
        {}
    }

    fn to_code_ir(&self) -> String {
        match self {
            Op::Sub => "-".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::test_util::test_code_ir;

    use super::*;

    #[test]
    fn test_valid_unary() {
        test_code_ir(
            "valid_unary",
            &Expression::Unary {
                op: Op::Sub,
                expr: Box::new(Wire::new(5, 6).into()),
            }
            .to_code_ir(),
        );
    }
}
