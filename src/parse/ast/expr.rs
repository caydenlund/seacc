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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::tests::AstBuilder;

    impl AstBuilder {
        fn push_expr(&self, expr: Expr) -> ExprId {
            self.exprs.borrow_mut().push(expr);
            #[allow(clippy::cast_possible_truncation)]
            ExprId((self.exprs.borrow().len() - 1) as u32)
        }

        pub fn expr_binary(&self, lhs: ExprId, op: BinaryOp, rhs: ExprId) -> ExprId {
            self.push_expr(Expr::Binary { lhs, op, rhs })
        }

        pub fn expr_unary(&self, op: UnaryOp, operand: ExprId) -> ExprId {
            self.push_expr(Expr::Unary { op, operand })
        }

        pub fn expr_int(&self, n: u64) -> ExprId {
            self.push_expr(Expr::Integer(n))
        }

        pub fn expr_float(&self, n: f64) -> ExprId {
            self.push_expr(Expr::Decimal(n))
        }

        pub fn expr_str(&self, s: impl Into<String>) -> ExprId {
            self.push_expr(Expr::String(s.into()))
        }

        pub fn expr_ident(&self, s: impl Into<String>) -> ExprId {
            self.push_expr(Expr::Ident(s.into()))
        }

        pub fn expr_mul(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Mul, rhs)
        }

        pub fn expr_mod(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Mod, rhs)
        }

        pub fn expr_div(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Div, rhs)
        }

        pub fn expr_add(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Add, rhs)
        }

        pub fn expr_sub(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Sub, rhs)
        }

        pub fn expr_set(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Assign, rhs)
        }

        pub fn expr_eq(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.expr_binary(lhs, BinaryOp::Eq, rhs)
        }

        pub fn expr_neg(&self, operand: ExprId) -> ExprId {
            self.expr_unary(UnaryOp::Negate, operand)
        }

        pub fn expr_not(&self, operand: ExprId) -> ExprId {
            self.expr_unary(UnaryOp::Not, operand)
        }

        pub fn expr_call(&self, callee: ExprId, args: &[ExprId]) -> ExprId {
            self.push_expr(Expr::Call {
                callee,
                args: args.into(),
            })
        }

        pub fn expr_ind(&self, obj: ExprId, ind: ExprId) -> ExprId {
            self.push_expr(Expr::GetIndex { obj, ind })
        }
    }
}
