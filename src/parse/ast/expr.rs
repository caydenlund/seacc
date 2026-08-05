use super::ExprId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Lshift,
    Rshift,
    BitAnd,
    BitOr,
    //
    Assign,
    MulAssign,
    DivAssign,
    ModAssign,
    AddAssign,
    SubAssign,
    LshiftAssign,
    RshiftAssign,
    BitAndAssign,
    BitOrAssign,
    //
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    LogicAnd,
    LogicOr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(u64),
    Decimal(f64),
    String(String),
    Ident(String),
    Binary {
        lhs: ExprId,
        op: BinaryOp,
        rhs: ExprId,
    },
    Unary {
        op: UnaryOp,
        operand: ExprId,
    },
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
    },
    GetIndex {
        obj: ExprId,
        ind: ExprId,
    },
}
