use crate::expr::Expr;

/// A statement.
#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub line: usize,  // The line of the source code from which the statement was derived.
    pub stmt_type: StmtType,  // The type of the statement.
}

/// Possible types of statements.
#[derive(Clone, Debug, PartialEq)]
pub enum StmtType {
    Block {
        body: Vec<Stmt>,
    },
    Break,
    Expression {
        expression: Expr,
    },
    Function {
        name: String,
        parameters: Vec<String>,
        body: Box<Stmt>,
    },
    If {
        condition: Expr,
        then_body: Box<Stmt>,
        else_body: Option<Box<Stmt>>,
    },
    Print {
        expression: Expr,
    },
    Return {
        expression: Expr,
    },
    VarDecl {
        name: String,
        value: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
