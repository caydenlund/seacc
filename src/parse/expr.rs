use crate::util::arena::{Arena, ArenaId};

pub type ExprId = ArenaId<Expr>;
pub type ExprArena = Arena<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Symbol(String),
    List(Vec<ExprId>),
    String(String),
    Bool(bool),
    Char(char),
}
