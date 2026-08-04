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
pub struct DeclId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(u32);

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Array(Box<Type>, usize),
    Struct {},
}

#[derive(Default, Clone)]
pub struct Ast {
    decls: Vec<Spanned<Decl>>,
    exprs: Vec<Spanned<Expr>>,
    items: Vec<Spanned<Item>>,
    stmts: Vec<Spanned<Stmt>>,
}
