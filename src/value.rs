use std::fmt;

use crate::{stmt::Stmt, hash_table::HashTable};

/// Represents evaluated/stored values within the interpreter.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String_(String),
    Bool(bool),
    Array(Vec<Value>),
    Dictionary(HashTable),
    Function {
        parameters: Vec<String>,
        body: Stmt,
    },
    BuiltinFunction(BuiltinFunction),
    Null,
}

impl Value {
    /// Returns the string of the `Value`'s type for error reports.
    pub fn type_to_string(&self) -> String {
        match self {
            Self::Number(..) => String::from("Number"),
            Self::String_(..) => String::from("String"),
            Self::Bool(..) => String::from("Boolean"),
            Self::Array(..) => String::from("Array"),
            Self::Dictionary(..) => String::from("Dictionary"),
            Self::Function {..} | Self::BuiltinFunction(..) => String::from("Function"),
            Self::Null => String::from("Null"),
        }
    }
}

/// Used when printing `Value`s.
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
            Self::Dictionary(dict) => {
                let flattened = dict.flatten();
                write!(f, "{{")?;
                let mut it = flattened.iter().peekable();
                while let Some(key_value) = it.next() {
                    key_value.key.fmt(f)?;
                    write!(f, ": ")?;
                    key_value.value.fmt(f)?;
                    if it.peek().is_some() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "}}")
            }
            Self::Function {..} | Self::BuiltinFunction(..) => write!(f, "<function>"),
            Self::Null => write!(f, "Null"),
        }
    }
}

/// Built-in functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuiltinFunction {
    Append,
    Input,
    Remove,
    Size,
    ToNumber,
    ToString,
}
