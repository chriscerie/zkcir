use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::node::{self, Node};

use super::{Expression, Ident, Value, VirtualWire, Wire};

extern crate alloc;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct ExprPath {
    /// {ident}::{ident}::...
    segments: Vec<Ident>,
}

impl Node for ExprPath {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        for segment in &mut self.segments {
            segment.visit_values(f);
        }
    }

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        for segment in &mut self.segments {
            segment.visit_virtual_wires(f);
        }
    }

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        for segment in &mut self.segments {
            segment.visit_wires(f);
        }
    }

    fn visit_expressions_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression,
    {
        for segment in &mut self.segments {
            segment.visit_expressions_mut(f);
        }
    }

    fn visit_idents_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident,
    {
        for segment in &mut self.segments {
            *segment = f(segment);
        }
    }

    fn to_code_ir(&self) -> alloc::string::String {
        self.segments
            .iter()
            .map(node::Node::to_code_ir)
            .collect::<Vec<_>>()
            .join("::")
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use crate::test_util::test_code_ir;

    use super::*;

    #[test]
    fn test_valid_unary() {
        test_code_ir(
            "valid_expr_path",
            &ExprPath {
                segments: vec![
                    Ident::String("left".to_string()),
                    Ident::Wire(Wire::new(5, 6)),
                ],
            }
            .to_code_ir(),
        );
    }
}
