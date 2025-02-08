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
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    ReturnStmt {
        keyword: Token,
        value: Option<Expr>,
    },
}

impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::ReturnStmt {
                keyword: _,
                value: _,
            } => todo!(),
            Self::Function {
                name: _,
                params: _,
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
