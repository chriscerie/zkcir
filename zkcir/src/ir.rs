extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;
use serde::Serialize;
use serde_json;

use crate::ast::Expression;
use crate::ast::Stmt;
use crate::ast::Value;
use crate::ast::VirtualWire;
use crate::ast::Wire;
use crate::node::Node;
use crate::END_DISCRIMINATOR;
use crate::START_DISCRIMINATOR;

/// Built circuit. This should be built using `CirBuilder` instead of creating this directly.
#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct Cir {
    pub config: Config,
    pub stmts: Vec<Stmt>,

    pub public_wire_inputs: Vec<Wire>,
    pub public_virtual_wire_inputs: Vec<VirtualWire>,
}

impl Cir {
    /// # Errors
    ///
    /// Errors from `serde_json::to_string_pretty`
    pub fn to_string(&self) -> Result<String, &'static str> {
        serde_json::to_string_pretty(&self).map_err(|_| "Failed serializing to json")
    }

    /// Same as `to_string` but omits random values. Useful for snapshot tests.
    ///
    /// # Errors
    ///
    /// Errors from `serde_json::to_string_pretty`
    pub fn to_string_omit_random(&self) -> Result<String, &'static str> {
        let mut new = self.clone();

        for expression in &mut new.stmts {
            expression.visit_wires(&mut |wire| {
                if let Some(Value::RandomU64(_)) = wire.value {
                    wire.value = Some(Value::Random);
                }
            });

            expression.visit_virtual_wires(&mut |wire| {
                if let Some(Value::RandomU64(_)) = wire.value {
                    wire.value = Some(Value::Random);
                }
            });
        }

        serde_json::to_string_pretty(&new).map_err(|_| "Failed serializing to json")
    }

    /// Appends discriminator to the start and end so zkcir's CLI can parse the output. You likely want `to_string`
    /// instead.
    ///
    /// # Errors
    ///
    /// Errors from `self.to_string()`
    pub fn to_cli_string(&self) -> Result<String, &'static str> {
        Ok(format!(
            "{START_DISCRIMINATOR}{}\n{END_DISCRIMINATOR}\n",
            self.to_string()?
        ))
    }

    pub fn to_code_ir(&self) -> String {
        self.stmts
            .iter()
            .map(Stmt::to_code_ir)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// # Errors
    ///
    /// Errors if cannot deserialize from json
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Config {
    num_wires: Option<u64>,
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone, Debug)]
pub struct CirBuilder {
    pub config: Config,
    pub stmts: Vec<Stmt>,

    pub public_wire_inputs: Vec<Wire>,
    pub public_virtual_wire_inputs: Vec<VirtualWire>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Operation {
    name: String,
    args: Vec<Expression>,
}

impl CirBuilder {
    #[must_use]
    pub fn new() -> Self {
        CirBuilder {
            config: Config { num_wires: None },
            stmts: Vec::new(),
            public_wire_inputs: Vec::new(),
            public_virtual_wire_inputs: Vec::new(),
        }
    }

    pub fn num_wires(&mut self, num: u64) -> &mut Self {
        self.config.num_wires = Some(num);
        self
    }

    pub fn add_stmt(&mut self, x: Stmt) -> &mut Self {
        self.stmts.push(x);
        self
    }

    pub fn set_virtual_wire_value(&mut self, index: usize, value: Value) -> &mut Self {
        for stmt in &mut self.stmts {
            stmt.visit_virtual_wires(&mut |virtual_wire| {
                if virtual_wire.index == index {
                    virtual_wire.value = Some(value);
                }
            });
        }

        for input in &mut self.public_virtual_wire_inputs {
            if input.index == index {
                input.value = Some(value);
            }
        }

        self
    }

    pub fn set_wire_value(&mut self, row: usize, column: usize, value: Value) -> &mut Self {
        for stmt in &mut self.stmts {
            stmt.visit_wires(&mut |wire| {
                if wire.row == row && wire.column == column {
                    wire.value = Some(value);
                }
            });
        }

        for input in &mut self.public_wire_inputs {
            if input.row == row && input.column == column {
                input.value = Some(value);
            }
        }

        self
    }

    #[must_use]
    pub fn build(&self) -> Cir {
        Cir {
            config: self.config,
            stmts: self.stmts.clone(),
            public_wire_inputs: self.public_wire_inputs.clone(),
            public_virtual_wire_inputs: self.public_virtual_wire_inputs.clone(),
        }
    }
}

impl Default for CirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use crate::{
        ast::{BinOp, Ident, VirtualWire, Wire},
        test_util::{test_code_ir, test_ir_string},
    };

    use super::*;

    #[test]
    fn test_valid_cir() {
        test_ir_string("valid_cir", CirBuilder::new().num_wires(10));
    }

    #[test]
    fn test_no_wires() {
        test_ir_string("test_no_wires", &CirBuilder::new());
    }

    #[test]
    fn test_binop() {
        let mut circuit = CirBuilder::new();
        circuit
            .add_stmt(Stmt::Local(
                Ident::Wire(Wire::new_private(3, 2)),
                Expression::BinaryOperator {
                    lhs: Box::new(Expression::BinaryOperator {
                        lhs: Box::new(Wire::new_private(1, 2).into()),
                        binop: BinOp::Add,
                        rhs: Box::new(VirtualWire::new_public(3).into()),
                    }),
                    binop: BinOp::Multiply,
                    rhs: Box::new(Wire::new_private(5, 6).into()),
                },
            ))
            .set_wire_value(5, 6, Value::U64(32))
            .set_virtual_wire_value(3, Value::U64(23));

        test_ir_string("test_binop", &circuit);
        test_code_ir("ir_binop", &circuit.build().to_code_ir());
    }

    #[test]
    fn test_verify() {
        test_ir_string(
            "test_verify",
            CirBuilder::new().add_stmt(Stmt::Verify(Expression::BinaryOperator {
                lhs: Box::new(Wire::new_private(5, 6).into()),
                binop: BinOp::Equal,
                rhs: Box::new(Wire::new_private(5, 6).into()),
            })),
        );
    }

    #[test]
    fn test_omit_random() {
        test_ir_string(
            "test_omit_random",
            CirBuilder::new()
                .add_stmt(Stmt::Local(
                    "rand".into(),
                    Expression::BinaryOperator {
                        lhs: Box::new(Expression::BinaryOperator {
                            lhs: Box::new(Wire::new_private(1, 2).into()),
                            binop: BinOp::Add,
                            rhs: Box::new(VirtualWire::new_public(3).into()),
                        }),
                        binop: BinOp::Multiply,
                        rhs: Box::new(Wire::new_private(5, 6).into()),
                    },
                ))
                .set_wire_value(5, 6, Value::RandomU64(32))
                .set_virtual_wire_value(3, Value::U64(23)),
        );
    }

    #[test]
    fn test_public_input() {
        test_code_ir(
            "ir_public_input",
            &CirBuilder::new()
                .add_stmt(Stmt::Local(
                    "num".into(),
                    Expression::BinaryOperator {
                        lhs: Box::new(Expression::BinaryOperator {
                            lhs: Box::new(Wire::new_public(1, 2).into()),
                            binop: BinOp::Add,
                            rhs: Box::new(VirtualWire::new_public(3).into()),
                        }),
                        binop: BinOp::Multiply,
                        rhs: Box::new(Wire::new_private(5, 6).into()),
                    },
                ))
                .build()
                .to_code_ir(),
        );
    }
}
