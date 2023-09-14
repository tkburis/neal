use crate::expr::Expr;

#[derive(Debug, PartialEq)]
pub struct Stmt {
    pub line: usize,
    pub stmt_type: StmtType,
}

#[derive(Debug, PartialEq)]
pub enum StmtType {
    Block {
        body: Vec<Stmt>,
    },
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
    VarDecl {
        name: String,
        value: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
