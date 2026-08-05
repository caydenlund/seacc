use super::{ExprId, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Variable {
        typ: Type,
        name: String,
        init: Option<ExprId>,
    },
    Function(FunctionDecl),
    Typedef {
        typ: Type,
        name: String,
    },
    Struct {
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Union {
        name: Option<String>,
        fields: Option<Vec<Field>>,
    },
    Enum {
        name: Option<String>,
        variants: Option<Vec<EnumVariant>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub ret_typ: Type,
    pub name: String,
    pub params: Vec<Param>,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub typ: Type,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub typ: Type,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<ExprId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ast::DeclId;
    use crate::parse::ast::tests::AstBuilder;

    impl AstBuilder {
        fn push_decl(&self, decl: Decl) -> DeclId {
            self.decls.borrow_mut().push(decl);
            #[allow(clippy::cast_possible_truncation)]
            DeclId((self.decls.borrow().len() - 1) as u32)
        }

        pub fn decl_fn(&self, func: FunctionDecl) -> DeclId {
            self.push_decl(Decl::Function(func))
        }

        pub fn decl_var(&self, typ: Type, name: impl Into<String>, init: Option<ExprId>) -> DeclId {
            self.push_decl(Decl::Variable {
                typ,
                name: name.into(),
                init,
            })
        }

        pub fn decl_type(&self, typ: Type, name: impl Into<String>) -> DeclId {
            self.push_decl(Decl::Typedef {
                typ,
                name: name.into(),
            })
        }

        pub fn decl_struct(
            &self,
            name: Option<impl Into<String>>,
            fields: Option<Vec<Field>>,
        ) -> DeclId {
            self.push_decl(Decl::Struct {
                name: name.map(|s| s.into()),
                fields,
            })
        }

        pub fn decl_union(
            &self,
            name: Option<impl Into<String>>,
            fields: Option<Vec<Field>>,
        ) -> DeclId {
            self.push_decl(Decl::Union {
                name: name.map(|s| s.into()),
                fields,
            })
        }

        pub fn decl_enum(
            &self,
            name: Option<impl Into<String>>,
            variants: Option<Vec<EnumVariant>>,
        ) -> DeclId {
            self.push_decl(Decl::Enum {
                name: name.map(|s| s.into()),
                variants,
            })
        }
    }
}
