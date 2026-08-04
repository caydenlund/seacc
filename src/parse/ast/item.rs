use super::{DeclId, FunctionDecl, StmtId};

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Decl(DeclId),
    FuncDef { decl: FunctionDecl, body: StmtId },
    Error,
}
