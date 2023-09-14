use std::collections::HashMap;

use crate::{token::Value, error::ErrorType};

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            panic!("Exited out of global scope.");
        }
    }

    pub fn declare(&mut self, name: String, value: &Value) {
        if let Some(last_scope) = self.scopes.last_mut() {
            last_scope.insert(name, value.clone());
        } else {
            panic!("No scopes to declare to.");
        }
    }

    // TODO: `Storable` trait for `Value` and functions.
    pub fn get(&self, name: String, index: Option<usize>, line: usize) -> Result<Value, ErrorType> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&name) {
                // If there is a value associated with `name`...
                if let Some(idx) = index {
                    // If an index is provided...
                    return match value {
                        Value::Array(elements) => {
                            if let Some(element) = elements.get(idx) {
                                Ok(element.clone())
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                Err(ErrorType::IndexError { name, index: idx, line })
                            }
                        },
                        // If an index is provided but the value is not an array...
                        _ => Err(ErrorType::NameNotIndexable { name, line }),
                    };
                } else {
                    // No index was provided.
                    return Ok(value.clone());
                }
            }
        }
        Err(ErrorType::NameError { name, line })
    }

    // TODO: element stuff
    pub fn assign(&mut self, name: String, index: Option<usize>, value: &Value, line: usize) -> Result<(), ErrorType> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(old_value) = scope.get_mut(&name) {
                // If there is a value associated with `name`...
                if let Some(idx) = index {
                    // If an index is provided...
                    return match old_value {
                        Value::Array(elements) => {
                            if idx < elements.len() {
                                elements[idx] = value.clone();
                                Ok(())
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                Err(ErrorType::IndexError { name, index: idx, line })
                            }
                        },
                        // If an index is provided but the value is not an array...
                        _ => Err(ErrorType::NameNotIndexable { name, line }),
                    }
                } else {
                    // No index was provided.
                    scope.insert(name, value.clone());
                }
                return Ok(());
            }
        }
        Err(ErrorType::NameError { name, line })
    }
}

#[cfg(test)]
mod tests {
    use crate::{token::Value, error::ErrorType};

    use super::Environment;

    #[test]
    fn one_scope() {
        //  var a = 5
        //  var b = [true, "hello world!"]
        //  b = "abc"
        let mut env = Environment::new();
        env.declare(String::from("a"), &Value::Number(5.0));
        env.declare(String::from("b"), &Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))]));
        assert_eq!(env.get(String::from("a"), None, 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))])));

        let _ = env.assign(String::from("b"), None, &Value::String_(String::from("abc")), 1);
        assert_eq!(env.get(String::from("a"), None, 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::String_(String::from("abc"))));
    }

    #[test]
    fn one_scope_element() {
        // var a = [1, true, "abc"]
        // "a[1] == true?"
        // a[0] = 5
        // "a[0] == 5?"
        // "a == [5, true, "abc"]?"
        let mut env = Environment::new();
        env.declare(String::from("a"), &Value::Array(vec![Value::Number(1.0), Value::Bool(true), Value::String_(String::from("abc"))]));
        assert_eq!(env.get(String::from("a"), Some(1), 1), Ok(Value::Bool(true)));

        let _ = env.assign(String::from("a"), Some(0), &Value::Number(5.0), 1);
        assert_eq!(env.get(String::from("a"), Some(0), 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("a"), None, 1), Ok(Value::Array(vec![Value::Number(5.0), Value::Bool(true), Value::String_(String::from("abc"))])));
    }

    #[test]
    fn many_scopes() {
        //  var a = 1
        //  var b = 2
        //  {
        //      a = 10
        //      "a == 10?"
        //      var b = 20
        //      "b == 20?"
        //      {
        //          b = 30
        //          "b == 30?"
        //      }
        //      "b == 30?"
        //  }
        //  "a == 10?"
        //  "b = 2?"
        let mut env = Environment::new();
        env.declare(String::from("a"), &Value::Number(1.0));
        env.declare(String::from("b"), &Value::Number(2.0));

        env.new_scope();
        let _ = env.assign(String::from("a"), None, &Value::Number(10.0), 1);
        env.declare(String::from("b"), &Value::Number(20.0));
        assert_eq!(env.get(String::from("a"), None, 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Number(20.0)));

        env.new_scope();
        let _ = env.assign(String::from("b"), None, &Value::Number(30.0), 1);
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Number(30.0)));

        env.exit_scope();
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Number(30.0)));

        env.exit_scope();
        assert_eq!(env.get(String::from("a"), None, 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Number(2.0)));
    }

    #[test]
    fn name_error_get() {
        let env = Environment::new();
        assert_eq!(env.get(String::from("b"), None, 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }

    #[test]
    fn name_error_assign() {
        let mut env = Environment::new();
        assert_eq!(env.assign(String::from("b"), None, &Value::Null, 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }

    #[test]
    fn declare_twice() {
        let mut env = Environment::new();
        env.declare(String::from("b"), &Value::Number(123.0));
        env.declare(String::from("b"), &Value::Number(55.0));
        assert_eq!(env.get(String::from("b"), None, 1), Ok(Value::Number(55.0)));
    }

    #[test]
    fn get_element_errors() {
        let mut env = Environment::new();
        env.declare(String::from("a"), &Value::Array(vec![Value::Number(1.0), Value::Bool(true), Value::String_(String::from("abc"))]));
        env.declare(String::from("b"), &Value::Number(123.0));
        assert_eq!(env.get(String::from("a"), Some(5), 1), Err(ErrorType::IndexError { name: String::from("a"), index: 5, line: 1 }));
        assert_eq!(env.get(String::from("b"), Some(5), 1), Err(ErrorType::NameNotIndexable { name: String::from("b"), line: 1 }));
    }
}
