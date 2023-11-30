use crate::{token, hash_table::KeyValue};

/// An expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub line: usize,
    pub expr_type: ExprType,
}

/// Possible types of expressions.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprType {
    Array {
        elements: Vec<Expr>,
    },
    Assignment {
        // Assignment is an expression for two reasons:
        // 1. Expressions like `a = b = 5` work;
        // 2. Calls have to be checked *before* assignments.
        // Otherwise, the statement `f()` will not work because we expect an `=` after the identifier.
        // Note `target` is an expression to allow both `Variable`s and `Element`s.
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Dictionary {
        elements: Vec<KeyValue<Expr>>,
    },
    Element {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Literal,
    },
    Unary {
        operator: token::Token,
        right: Box<Expr>,
    },
    Variable {
        name: String,
    },
}
