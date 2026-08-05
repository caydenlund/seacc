use super::{DeclId, FunctionDecl, StmtId};

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Decl(DeclId),
    FuncDef {
        decl: FunctionDecl,
        body: Vec<StmtId>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::ItemId;
    use crate::parse::ast::tests::AstBuilder;

    impl AstBuilder {
        fn push_item(&self, item: Item) -> ItemId {
            self.items.borrow_mut().push(item);
            #[allow(clippy::cast_possible_truncation)]
            ItemId((self.items.borrow().len() - 1) as u32)
        }

        pub fn item_decl(&self, decl: DeclId) -> ItemId {
            self.push_item(Item::Decl(decl))
        }

        pub fn item_func(&self, decl: FunctionDecl, body: &[StmtId]) -> ItemId {
            self.push_item(Item::FuncDef {
                decl,
                body: body.into(),
            })
        }
    }
}
