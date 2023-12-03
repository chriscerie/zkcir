extern crate alloc;

use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};

use crate::node::Node;

use super::expr::{VirtualWire, Wire};

/// Identifier. `x` in `let x = y;`
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub enum Ident {
    String(String),

    // Wires act like variables
    Wire(Wire),
    VirtualWire(VirtualWire),
}

impl Node for Ident {
    fn visit_values<F>(&mut self, _f: &mut F)
    where
        F: FnMut(&mut super::expr::Value),
    {
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

    fn to_code_ir(&self) -> String {
        match self {
            Ident::String(ident) => ident.clone(),
            Ident::Wire(wire) => wire.to_code_ir(),
            Ident::VirtualWire(virtual_wire) => virtual_wire.to_code_ir(),
        }
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Ident::String(s.to_string())
    }
}
