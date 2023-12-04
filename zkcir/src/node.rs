extern crate alloc;

use alloc::string::String;

use crate::ast::{Expression, Ident, Value, VirtualWire, Wire};

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

    /// Callback returns new expression
    fn visit_expressions_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Expression) -> Expression;

    /// Callback returns new expression
    fn visit_idents_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Ident) -> Ident;

    #[must_use]
    fn to_code_ir(&self) -> String;
}
