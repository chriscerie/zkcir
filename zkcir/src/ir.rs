extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Serialize;
use serde_json;

use crate::ast::Expression;
use crate::ast::Value;
use crate::END_DISCRIMINATOR_JSON;
use crate::END_DISCRIMINATOR_SOURCE;
use crate::START_DISCRIMINATOR_JSON;
use crate::START_DISCRIMINATOR_SOURCE;

#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
pub struct Config {
    num_wires: Option<u64>,
}

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct CirBuilder {
    pub config: Config,
    pub expressions: Vec<Expression>,
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
            expressions: Vec::new(),
        }
    }

    pub fn num_wires(&mut self, num: u64) -> &mut Self {
        self.config.num_wires = Some(num);
        self
    }

    pub fn add_expression(&mut self, x: Expression) -> &mut Self {
        self.expressions.push(x);
        self
    }

    pub fn set_virtual_wire_value(&mut self, index: usize, value: Value) -> &mut Self {
        for expression in &mut self.expressions {
            expression.visit_virtual_wires(&mut |virtual_wire| {
                if virtual_wire.index == index {
                    virtual_wire.value = Some(value);
                }
            });
        }
        self
    }

    pub fn set_wire_value(&mut self, row: usize, column: usize, value: Value) -> &mut Self {
        for expression in &mut self.expressions {
            expression.visit_wires(&mut |wire| {
                if wire.row == row && wire.column == column {
                    wire.value = Some(value);
                }
            });
        }
        self
    }

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

        for expression in &mut new.expressions {
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
            "{START_DISCRIMINATOR_JSON}{}\n{END_DISCRIMINATOR_JSON}\n{START_DISCRIMINATOR_SOURCE}{}\n{END_DISCRIMINATOR_SOURCE}\n",
            self.to_string()?, self.to_code_ir()
        ))
    }

    #[must_use]
    pub fn to_code_ir(&self) -> String {
        self.expressions
            .iter()
            .map(Expression::to_code_ir)
            .collect::<Vec<_>>()
            .join("\n\n")
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
        ast::{BinOp, VirtualWire, Wire},
        test_util::test_ir_string,
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
        test_ir_string(
            "test_binop",
            CirBuilder::new()
                .add_expression(Expression::BinaryOperator {
                    lhs: Box::new(Expression::BinaryOperator {
                        lhs: Box::new(Wire::new(1, 2).into()),
                        binop: BinOp::Add,
                        rhs: Box::new(VirtualWire::new(3).into()),
                        result: None,
                    }),
                    binop: BinOp::Multiply,
                    rhs: Box::new(Wire::new(5, 6).into()),
                    result: None,
                })
                .set_wire_value(5, 6, Value::U64(32))
                .set_virtual_wire_value(3, Value::U64(23)),
        );
    }

    #[test]
    fn test_verify() {
        test_ir_string(
            "test_verify",
            CirBuilder::new().add_expression(Expression::Verify(Box::new(
                Expression::BinaryOperator {
                    lhs: Box::new(Wire::new(5, 6).into()),
                    binop: BinOp::Equal,
                    rhs: Box::new(Wire::new(5, 6).into()),
                    result: Some(Box::new(Wire::new(10, 11).into())),
                },
            ))),
        );
    }

    #[test]
    fn test_omit_random() {
        test_ir_string(
            "test_omit_random",
            CirBuilder::new()
                .add_expression(Expression::BinaryOperator {
                    lhs: Box::new(Expression::BinaryOperator {
                        lhs: Box::new(Wire::new(1, 2).into()),
                        binop: BinOp::Add,
                        rhs: Box::new(VirtualWire::new(3).into()),
                        result: None,
                    }),
                    binop: BinOp::Multiply,
                    rhs: Box::new(Wire::new(5, 6).into()),
                    result: None,
                })
                .set_wire_value(5, 6, Value::RandomU64(32))
                .set_virtual_wire_value(3, Value::U64(23)),
        );
    }
}
