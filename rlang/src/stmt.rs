use crate::{expr::Expr, lexer::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    Block {
        statements: Vec<Box<Stmt>>,
    },
    IfStmt {
        predicate: Expr,
        then: Box<Stmt>,
        r#else: Option<Box<Stmt>>,
    },
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,
    },
    ForStmt {
        var_decl: Option<Box<Expr>>,
        expr_stmt: Option<Box<Stmt>>,

        condition: Expr,
        then: Option<Expr>,

        body: Box<Stmt>,
    },
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::ForStmt {
                var_decl,
                expr_stmt,
                condition,
                then,
                body: _,
            } => todo!(),
            Self::WhileStmt {
                condition: _,
                body: _,
            } => todo!(),
            Self::IfStmt {
                predicate: _,
                then: _,
                r#else: _,
            } => todo!(),
            Self::Block { statements } => {
                format!(
                    "(block {})",
                    statements
                        .iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            Self::Expression { expression } => expression.to_string(),
            Self::Print { expression } => format!("(print {})", expression.to_string()),
            Self::Var {
                name,
                initializer: _,
            } => format!("(var {})", name.lexme),
        };
        write!(f, "{}", s)
    }
}
