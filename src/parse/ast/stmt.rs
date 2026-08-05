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
    Error,
}
