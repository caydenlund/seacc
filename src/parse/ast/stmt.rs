use super::{DeclId, ExprId, StmtId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Decl(DeclId),
    Expr(ExprId),
    Block(Vec<StmtId>),
    If {
        cond: ExprId,
        branch_then: StmtId,
        branch_else: Option<StmtId>,
    },
    While {
        cond: ExprId,
        body: StmtId,
    },
    Return(Option<ExprId>),
    Break,
    Continue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::tests::AstBuilder;

    impl AstBuilder {
        fn push_stmt(&self, stmt: Stmt) -> StmtId {
            self.stmts.borrow_mut().push(stmt);
            #[allow(clippy::cast_possible_truncation)]
            StmtId((self.stmts.borrow().len() - 1) as u32)
        }

        pub fn stmt_decl(&self, decl: DeclId) -> StmtId {
            self.push_stmt(Stmt::Decl(decl))
        }

        pub fn stmt_expr(&self, expr: ExprId) -> StmtId {
            self.push_stmt(Stmt::Expr(expr))
        }

        pub fn stmt_block(&self, stmts: &[StmtId]) -> StmtId {
            self.push_stmt(Stmt::Block(stmts.to_vec()))
        }

        pub fn stmt_if(
            &self,
            cond: ExprId,
            branch_then: StmtId,
            branch_else: Option<StmtId>,
        ) -> StmtId {
            self.push_stmt(Stmt::If {
                cond,
                branch_then,
                branch_else,
            })
        }

        pub fn stmt_while(&self, cond: ExprId, body: StmtId) -> StmtId {
            self.push_stmt(Stmt::While { cond, body })
        }

        pub fn stmt_return(&self, value: Option<ExprId>) -> StmtId {
            self.push_stmt(Stmt::Return(value))
        }

        pub fn stmt_break(&self) -> StmtId {
            self.push_stmt(Stmt::Break)
        }

        pub fn stmt_continue(&self) -> StmtId {
            self.push_stmt(Stmt::Continue)
        }
    }
}
