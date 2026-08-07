use seacc::parse::ast::{
    Ast, Decl, DeclId, Expr, ExprId, FunctionDecl, Item, Param, Stmt, StmtId, Type, UnaryOp,
};

struct Tree {
    label: String,
    children: Vec<Self>,
}

impl Tree {
    fn leaf(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    fn branch(label: impl Into<String>, children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            label: label.into(),
            children: children.into_iter().collect(),
        }
    }

    fn print(&self) {
        println!("{}", self.label);
        self.print_children("");
    }

    fn print_children(&self, prefix: &str) {
        for (index, child) in self.children.iter().enumerate() {
            let last = index + 1 == self.children.len();
            let connector = if last { "└─" } else { "├─" };
            println!("{prefix}{connector}{}", child.label);
            child.print_children(&format!("{prefix}{}", if last { "  " } else { "│ " }));
        }
    }
}

fn type_tree(typ: &Type) -> Tree {
    match typ {
        Type::Int => Tree::leaf("Type: int"),
        Type::Float => Tree::leaf("Type: float"),
        Type::Array(element, length) => Tree::branch(
            "Type: Array",
            [type_tree(element), Tree::leaf(format!("Length: {length}"))],
        ),
        Type::Struct {} => Tree::leaf("Type: struct"),
    }
}

fn param_tree(Param { typ, name }: &Param) -> Tree {
    let label = match name {
        Some(name) => format!("{} {name}", type_name(typ)),
        None => type_name(typ).to_owned(),
    };
    Tree::leaf(label)
}

fn type_name(typ: &Type) -> &str {
    match typ {
        Type::Int => "int",
        Type::Float => "float",
        Type::Array(_, _) => "Array",
        Type::Struct {} => "struct",
    }
}

fn labeled_expr(ast: &Ast, label: &str, expr: ExprId) -> Tree {
    let Tree {
        label: expr_label,
        children,
    } = expr_tree(ast, expr);
    Tree {
        label: format!("{label}: {expr_label}"),
        children,
    }
}

fn function_children(def: &FunctionDecl) -> Vec<Tree> {
    let mut children = vec![
        Tree::leaf(format!("Return Type: {}", type_name(&def.ret_typ))),
        Tree::leaf(format!("Name: {}", def.name)),
        Tree::branch(
            format!("Params: [{}]", def.params.len()),
            def.params.iter().map(param_tree),
        ),
    ];
    if def.variadic {
        children.push(Tree::leaf("Variadic"));
    }
    children
}

fn decl_tree(ast: &Ast, id: DeclId) -> Tree {
    match &ast.decl(id) {
        Decl::Variables { typ, vars } => Tree::branch(
            "Decl: Variables",
            std::iter::once(type_tree(typ)).chain(vars.iter().map(|(name, init)| {
                let mut children = vec![Tree::leaf(format!("Name: {name}"))];
                if let Some(init) = init {
                    children.push(labeled_expr(ast, "Value", *init));
                }
                Tree::branch("Variable", children)
            })),
        ),
        Decl::Function(def) => Tree::branch("Decl: Func", function_children(def)),
        Decl::Typedef { typ, name } => Tree::branch(
            "Decl: Typedef",
            [type_tree(typ), Tree::leaf(format!("Name: {name}"))],
        ),
        Decl::Struct { name, fields } => Tree::branch(
            "Decl: Struct",
            name.iter()
                .map(|name| Tree::leaf(format!("Name: {name}")))
                .chain(fields.iter().flatten().map(|field| {
                    Tree::leaf(match &field.name {
                        Some(name) => format!("{} {name}", type_name(&field.typ)),
                        None => type_name(&field.typ).to_owned(),
                    })
                })),
        ),
        Decl::Union { name, fields } => Tree::branch(
            "Decl: Union",
            name.iter()
                .map(|name| Tree::leaf(format!("Name: {name}")))
                .chain(fields.iter().flatten().map(|field| {
                    Tree::leaf(format!(
                        "{} {}",
                        type_name(&field.typ),
                        field.name.as_deref().unwrap_or("")
                    ))
                })),
        ),
        Decl::Enum { name, variants } => Tree::branch(
            "Decl: Enum",
            name.iter()
                .map(|name| Tree::leaf(format!("Name: {name}")))
                .chain(
                    variants
                        .iter()
                        .flatten()
                        .map(|variant| Tree::leaf(variant.name.clone())),
                ),
        ),
    }
}

fn expr_tree(ast: &Ast, id: ExprId) -> Tree {
    match &ast.expr(id) {
        Expr::Integer(n) => Tree::leaf(n.to_string()),
        Expr::Decimal(n) => Tree::leaf(n.to_string()),
        Expr::String(s) => Tree::leaf(format!(r#""{s}""#)),
        Expr::Ident(name) => Tree::leaf(name),
        Expr::Binary { lhs, op, rhs } => Tree::branch(
            format!("{op:?}"),
            [expr_tree(ast, *lhs), expr_tree(ast, *rhs)],
        ),
        Expr::Unary { op, operand } => Tree::branch(unary_name(*op), [expr_tree(ast, *operand)]),
        Expr::Call { callee, args } => Tree::branch(
            "Call",
            [
                labeled_expr(ast, "Func", *callee),
                Tree::branch(
                    format!("Params: [{}]", args.len()),
                    args.iter().map(|arg| expr_tree(ast, *arg)),
                ),
            ],
        ),
        Expr::GetIndex { obj, ind } => {
            Tree::branch("Index", [expr_tree(ast, *obj), expr_tree(ast, *ind)])
        }
    }
}

fn unary_name(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "Not",
        UnaryOp::Negate => "Negate",
    }
}

fn stmt_tree(ast: &Ast, id: StmtId) -> Tree {
    match &ast.stmt(id) {
        Stmt::Decl(id) => decl_tree(ast, *id),
        Stmt::Expr(id) => expr_tree(ast, *id),
        Stmt::Block(ids) => Tree::branch(
            format!("Stmts: [{}]", ids.len()),
            ids.iter().map(|id| stmt_tree(ast, *id)),
        ),
        Stmt::If {
            cond,
            branch_then,
            branch_else,
        } => {
            let mut children = vec![
                labeled_expr(ast, "Condition", *cond),
                Tree::branch("Then", [stmt_tree(ast, *branch_then)]),
            ];
            if let Some(branch_else) = branch_else {
                children.push(Tree::branch("Else", [stmt_tree(ast, *branch_else)]));
            }
            Tree::branch("If", children)
        }
        Stmt::While { cond, body } => Tree::branch(
            "While",
            [
                labeled_expr(ast, "Condition", *cond),
                Tree::branch("Body", [stmt_tree(ast, *body)]),
            ],
        ),
        Stmt::Return(expr) => match expr {
            Some(expr) => labeled_expr(ast, "Return", *expr),
            None => Tree::leaf("Return"),
        },
        Stmt::Break => Tree::leaf("Break"),
        Stmt::Continue => Tree::leaf("Continue"),
    }
}

fn item_tree(ast: &Ast, item: &Item) -> Tree {
    match item {
        Item::Decl(id) => decl_tree(ast, *id),
        Item::FuncDef { decl, body } => {
            let mut children = function_children(decl);
            let body = Tree::branch(
                format!("Stmts: [{}]", body.len()),
                body.iter().map(|s| stmt_tree(ast, *s)),
            );
            children.push(body);
            Tree::branch("Def: Func", children)
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let prgm = &args[0];
    if args.len() != 2 {
        eprintln!("usage: {prgm} <sourcefile>");
        std::process::exit(2);
    }
    let contents = std::fs::read_to_string(&args[1])?;
    let tokens = match seacc::lex::lex(0, &contents).0 {
        Ok(tokens) => tokens,
        Err(e) => panic!("unable to tokenize source file: {e:?}"),
    };
    let ast = seacc::parse::parse(&tokens).ast;

    for (index, item) in ast.items.iter().enumerate() {
        if index != 0 {
            println!();
        }
        item_tree(&ast, &item.value).print();
    }

    Ok(())
}
