extern crate alloc;

use alloc::string::String;

use crate::ast::expr::{Value, VirtualWire, Wire};

pub trait Node {
    fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value);

    fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire);

    fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire);

    #[must_use]
    fn to_code_ir(&self) -> String;
}
