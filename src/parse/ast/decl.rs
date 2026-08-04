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
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    ret_typ: Type,
    name: String,
    params: Vec<Param>,
    variadic: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Option<String>,
    pub value: Option<ExprId>,
}
