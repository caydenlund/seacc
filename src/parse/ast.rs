pub type ExprId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Add,
    Sub,
    Lshift,
    Rshift,
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(u64),
    Decimal(f64),
    String(String),
    Ident(String),
    BinaryOp(ExprId, BinaryOp, ExprId),
    UnaryOp(UnaryOp, ExprId),
}

impl Expr {
    #[must_use]
    pub fn sexpr(&self, exprs: &[Self]) -> String {
        match self {
            Self::Integer(i) => i.to_string(),
            Self::Decimal(d) => d.to_string(),
            Self::String(s) => format!(
                "\"{}\"",
                s.chars()
                    .map(|ch| match ch {
                        '\n' => "\\n".into(),
                        '\t' => "\\t".into(),
                        '\r' => "\\r".into(),
                        '\\' => "\\\\".into(),
                        '"' => "\\\"".into(),
                        _ => ch.to_string(),
                    })
                    .collect::<String>()
            ),
            Self::Ident(i) => format!("(Ident {i})"),
            Self::BinaryOp(lop, binary_op, rop) => format!(
                "({binary_op:?} {} {})",
                exprs[*lop].sexpr(exprs),
                exprs[*rop].sexpr(exprs)
            ),
            Self::UnaryOp(unary_op, op) => format!("({unary_op:?} {})", exprs[*op].sexpr(exprs)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default, Clone)]
    struct ExprBuilder {
        exprs: RefCell<Vec<Expr>>,
    }

    impl ExprBuilder {
        fn push(&self, expr: Expr) -> ExprId {
            self.exprs.borrow_mut().push(expr);
            self.exprs.borrow().len() - 1
        }

        fn int(&self, val: u64) -> ExprId {
            self.push(Expr::Integer(val))
        }

        fn mul(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.push(Expr::BinaryOp(lhs, BinaryOp::Mul, rhs))
        }

        fn div(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.push(Expr::BinaryOp(lhs, BinaryOp::Div, rhs))
        }

        fn add(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.push(Expr::BinaryOp(lhs, BinaryOp::Add, rhs))
        }

        fn sub(&self, lhs: ExprId, rhs: ExprId) -> ExprId {
            self.push(Expr::BinaryOp(lhs, BinaryOp::Sub, rhs))
        }

        fn ident(&self, s: &str) -> ExprId {
            self.push(Expr::Ident(s.into()))
        }

        fn finish(self) -> Vec<Expr> {
            self.exprs.into_inner()
        }
    }

    fn sexpr(exprs: &[Expr]) -> String {
        exprs.last().unwrap().sexpr(exprs)
    }

    #[test]
    fn builder() {
        let eb = ExprBuilder::default();
        assert_eq!(eb.clone().finish(), vec![]);
        assert_eq!(eb.int(100), 0);
        assert_eq!(eb.clone().finish(), vec![Expr::Integer(100)]);
        assert_eq!(eb.int(200), 1);
        assert_eq!(
            eb.clone().finish(),
            vec![Expr::Integer(100), Expr::Integer(200)]
        );
        assert_eq!(eb.mul(0, 1), 2);
        assert_eq!(
            eb.clone().finish(),
            vec![
                Expr::Integer(100),
                Expr::Integer(200),
                Expr::BinaryOp(0, BinaryOp::Mul, 1)
            ]
        );
        assert_eq!(eb.mul(eb.ident("foo"), eb.ident("bar")), 5);
    }

    #[test]
    fn sexprs() {
        let eb = ExprBuilder::default();
        eb.add(eb.ident("foo"), eb.mul(eb.int(1), eb.int(2)));
        assert_eq!(sexpr(&eb.finish()), "(Add (Ident foo) (Mul 1 2))");

        let eb = ExprBuilder::default();
        eb.sub(eb.int(9), eb.div(eb.int(6), eb.int(2)));
        assert_eq!(sexpr(&eb.finish()), "(Sub 9 (Div 6 2))");
    }
}
