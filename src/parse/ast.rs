use crate::{Span, Spanned};

mod decl;
pub use decl::{Decl, EnumVariant, Field, FunctionDecl, Param};

mod expr;
pub use expr::{BinaryOp, Expr, UnaryOp};

mod item;
pub use item::Item;

mod stmt;
pub use stmt::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclId(pub(self) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub(self) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub(self) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StmtId(pub(self) u32);

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
    pub fn decl(&self, id: DeclId) -> &Decl {
        &self.decls[id.0 as usize].value
    }

    #[must_use]
    pub fn decl_span(&self, id: DeclId) -> Span {
        self.decls[id.0 as usize].span
    }

    pub(crate) fn push_decl(&mut self, decl: Decl, span: Span) -> DeclId {
        self.decls.push(Spanned { value: decl, span });
        #[allow(clippy::cast_possible_truncation)]
        DeclId((self.decls.len() - 1) as u32)
    }

    #[must_use]
    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize].value
    }

    #[must_use]
    pub fn expr_span(&self, id: ExprId) -> Span {
        self.exprs[id.0 as usize].span
    }

    pub(crate) fn push_expr(&mut self, expr: Expr, span: Span) -> ExprId {
        self.exprs.push(Spanned { value: expr, span });
        #[allow(clippy::cast_possible_truncation)]
        ExprId((self.exprs.len() - 1) as u32)
    }

    #[must_use]
    pub fn item(&self, id: ItemId) -> &Item {
        &self.items[id.0 as usize].value
    }

    #[must_use]
    pub fn item_span(&self, id: ItemId) -> Span {
        self.items[id.0 as usize].span
    }

    pub(crate) fn push_item(&mut self, item: Item, span: Span) -> ItemId {
        self.items.push(Spanned { value: item, span });
        #[allow(clippy::cast_possible_truncation)]
        ItemId((self.items.len() - 1) as u32)
    }

    #[must_use]
    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize].value
    }

    #[must_use]
    pub fn stmt_span(&self, id: StmtId) -> Span {
        self.stmts[id.0 as usize].span
    }

    pub(crate) fn push_stmt(&mut self, stmt: Stmt, span: Span) -> StmtId {
        self.stmts.push(Spanned { value: stmt, span });
        #[allow(clippy::cast_possible_truncation)]
        StmtId((self.stmts.len() - 1) as u32)
    }
}

#[cfg(test)]
pub mod tests {
    use std::cell::RefCell;

    use super::*;

    #[derive(Default, Clone)]
    pub struct AstBuilder {
        pub decls: RefCell<Vec<Decl>>,
        pub exprs: RefCell<Vec<Expr>>,
        pub items: RefCell<Vec<Item>>,
        pub stmts: RefCell<Vec<Stmt>>,
    }

    impl AstBuilder {
        pub fn assert_matches(&self, ast: &Ast) {
            assert_eq!(
                ast.decls.iter().map(|decl| &decl.value).collect::<Vec<_>>(),
                self.decls.borrow().iter().collect::<Vec<_>>(),
            );
            assert_eq!(
                ast.exprs.iter().map(|expr| &expr.value).collect::<Vec<_>>(),
                self.exprs.borrow().iter().collect::<Vec<_>>(),
            );
            assert_eq!(
                ast.items.iter().map(|item| &item.value).collect::<Vec<_>>(),
                self.items.borrow().iter().collect::<Vec<_>>(),
            );
            assert_eq!(
                ast.stmts.iter().map(|stmt| &stmt.value).collect::<Vec<_>>(),
                self.stmts.borrow().iter().collect::<Vec<_>>(),
            );
        }
    }
}
