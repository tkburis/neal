use std::fmt;

use crate::stmt::Stmt;

/// Represents evaluated/stored values within the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    // Dictionary(HashTable),
    String_(String),
    Bool(bool),
    Array(Vec<Value>),
    Function {
        parameters: Vec<String>,
        body: Stmt,
    },
    Null,
}

impl Value {
    pub fn type_to_string(&self) -> String {
        match self {
            Self::Number(..) => String::from("Number"),
            Self::String_(..) => String::from("String"),
            Self::Bool(..) => String::from("Boolean"),
            Self::Array(..) => String::from("Array"),
            Self::Function {..} => String::from("Function"),
            Self::Null => String::from("Null"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(x) => write!(f, "{}", x),
            Self::String_(x) => write!(f, "{}", x),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Array(array) => {
                write!(f, "[")?;
                let mut it = array.iter().peekable();
                while let Some(x) = it.next() {
                    x.fmt(f)?;
                    if it.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            },
            Self::Function {..} => write!(f, "<function>"),
            Self::Null => write!(f, "Null"),
        }
    }
}
