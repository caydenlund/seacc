use crate::Spanned;

mod decl;
pub use decl::{Decl, EnumVariant, Field, FunctionDecl, Param};

mod expr;
pub use expr::{BinaryOp, Expr, UnaryOp};

mod item;
pub use item::Item;

mod stmt;
pub use stmt::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Array(Box<Self>, usize),
    Struct {},
}

#[derive(Default, Clone)]
pub struct Ast {
    pub decls: Vec<Spanned<Decl>>,
    pub exprs: Vec<Spanned<Expr>>,
    pub items: Vec<Spanned<Item>>,
    pub stmts: Vec<Spanned<Stmt>>,
}

impl Ast {
    #[must_use]
    pub fn decl(&self, id: DeclId) -> &Spanned<Decl> {
        &self.decls[id.0 as usize]
    }

    #[must_use]
    pub fn expr(&self, id: ExprId) -> &Spanned<Expr> {
        &self.exprs[id.0 as usize]
    }

    #[must_use]
    pub fn item(&self, id: ItemId) -> &Spanned<Item> {
        &self.items[id.0 as usize]
    }

    #[must_use]
    pub fn stmt(&self, id: StmtId) -> &Spanned<Stmt> {
        &self.stmts[id.0 as usize]
    }
}
