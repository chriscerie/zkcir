use serde::Serialize;

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
        result: Option<Box<Expression>>,
    },

    Int(i64),

    // Wrapped value is the id
    Random(usize),

    Verify(Box<Expression>),
    VirtualWire(VirtualWire),
    Wire(Wire),
}

impl Expression {
    pub fn visit_values<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Value),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_values(f);
                rhs.visit_values(f);

                if let Some(result) = result {
                    result.visit_values(f);
                }
            }
            Expression::Verify(expr) => {
                expr.visit_values(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                if let Some(value) = &mut virtual_wire.value {
                    f(value);
                }
            }
            Expression::Wire(wire) => {
                if let Some(value) = &mut wire.value {
                    f(value);
                }
            }
            Expression::Int(_) | Expression::Random(_) => {}
        }
    }

    pub fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_virtual_wires(f);
                rhs.visit_virtual_wires(f);

                if let Some(result) = result {
                    result.visit_virtual_wires(f);
                }
            }
            Expression::Verify(expr) => {
                expr.visit_virtual_wires(f);
            }
            Expression::VirtualWire(virtual_wire) => {
                f(virtual_wire);
            }
            Expression::Wire(_) | Expression::Int(_) | Expression::Random(_) => {}
        }
    }

    pub fn visit_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut Wire),
    {
        match self {
            Expression::BinaryOperator {
                lhs, rhs, result, ..
            } => {
                lhs.visit_wires(f);
                rhs.visit_wires(f);

                if let Some(result) = result {
                    result.visit_wires(f);
                }
            }
            Expression::Verify(expr) => {
                expr.visit_wires(f);
            }
            Expression::Wire(wire) => {
                f(wire);
            }
            Expression::VirtualWire { .. } | Expression::Int(_) | Expression::Random(_) => {}
        }
    }
}

#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
pub enum Value {
    U64(u64),
    RandomU64(u64),

    /// Enables generating deterministic IRs even when using random values. Useful for snapshot tests
    Random,
}

/// `VirtualTarget` in plonky2
#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
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

impl From<VirtualWire> for Expression {
    fn from(val: VirtualWire) -> Self {
        Expression::VirtualWire(val)
    }
}

/// `Target` in plonky2
#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
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

impl From<Wire> for Expression {
    fn from(val: Wire) -> Self {
        Expression::Wire(val)
    }
}

#[derive(PartialEq, Eq, Serialize, Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Divide,
    Equal,
    Exponent,
    GreaterThanEqual,
    GreaterThan,
    LessThan,
    LessThanEqual,
    Multiply,
    Subtract,
}
