use serde::Serialize;

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum Expression {
    BinaryOperator {
        lhs: Box<Expression>,
        binop: BinOp,
        rhs: Box<Expression>,
        result: Option<WireOrVirtualWire>,
    },

    Int(i64),

    // Wrapped value is the id
    Random(usize),

    Verify(Box<Expression>),
    VirtualWire(VirtualWire),
    Wire(Wire),
}

impl Expression {
    pub fn visit_virtual_wires<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut VirtualWire),
    {
        match self {
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_virtual_wires(f);
                rhs.visit_virtual_wires(f);
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
            Expression::BinaryOperator { lhs, rhs, .. } => {
                lhs.visit_wires(f);
                rhs.visit_wires(f);
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

#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub enum WireOrVirtualWire {
    Wire(Wire),
    VirtualWire(VirtualWire),
}

/// `VirtualTarget` in plonky2
#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct VirtualWire {
    pub index: usize,
    pub value: Option<u64>,
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
#[derive(PartialEq, Eq, Serialize, Clone, Debug)]
pub struct Wire {
    pub row: usize,
    pub column: usize,
    pub value: Option<u64>,
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
