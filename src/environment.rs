use std::collections::HashMap;

use crate::{value::Value, error::ErrorType};

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}
// TODO: BUILTIN FUNCTIONS like append, input (https://users.rust-lang.org/t/how-to-get-user-input/5176/8)

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

    pub fn get(&self, name: String, index: Option<Value>, line: usize) -> Result<Value, ErrorType> {
        for scope in self.scopes.iter().rev() {
            if let Some(object) = scope.get(&name) {
                // If there is a value associated with `name`...
                if let Some(key) = index {
                    // If an index is provided...
                    return match object {
                        Value::Array(array) => {
                            let idx = index_value_to_usize(&key, line)?;
                            if let Some(element) = array.get(idx) {
                                Ok(element.clone())
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                Err(ErrorType::OutOfBoundsIndexError { name: Some(name), index: idx, line })
                            }
                        },
                        Value::Dictionary(dict) => {
                            dict.get(&key, line)
                        },
                        // If an index is provided but the value is not an array...
                        _ => Err(ErrorType::NotIndexableError { name: Some(name), line }),
                    };
                } else {
                    // No index was provided.
                    return Ok(object.clone());
                }
            }
        }
        Err(ErrorType::NameError { name, line })
    }

    pub fn assign(&mut self, name: String, index: Option<&Value>, value: &Value, line: usize) -> Result<(), ErrorType> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(object) = scope.get_mut(&name) {
                // If there is a value associated with `name`...
                if let Some(key) = index {
                    // If an index is provided...
                    return match object {
                        Value::Array(array) => {
                            let idx = index_value_to_usize(&key, line)?;
                            if idx < array.len() {
                                array[idx] = value.clone();
                                Ok(())
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                Err(ErrorType::OutOfBoundsIndexError { name: Some(name), index: idx, line })
                            }
                        },
                        Value::Dictionary(dict) => {
                            dict.insert(&key, value, line)
                        }
                        // If an index is provided but the value is not an array...
                        _ => Err(ErrorType::NotIndexableError { name: Some(name), line }),
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

pub fn index_value_to_usize(index: &Value, line: usize) -> Result<usize, ErrorType> {
    match index {
        Value::Number(index_num) => {
            if *index_num >= 0.0 && index_num.fract() == 0.0 {
                Ok(*index_num as usize)
            } else {
                Err(ErrorType::NonNaturalIndexError { got: index.clone(), line })
            }
        },
        _ => Err(ErrorType::NonNumberIndexError { got: index.type_to_string(), line })
    }
}

#[cfg(test)]
mod tests {
    use crate::{value::Value, error::ErrorType};

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
        assert_eq!(env.get(String::from("a"), Some(Value::Number(1.0)), 1), Ok(Value::Bool(true)));

        let _ = env.assign(String::from("a"), Some(&Value::Number(0.0)), &Value::Number(5.0), 1);
        assert_eq!(env.get(String::from("a"), Some(Value::Number(0.0)), 1), Ok(Value::Number(5.0)));
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
        assert_eq!(env.get(String::from("a"), Some(Value::Number(5.0)), 1), Err(ErrorType::OutOfBoundsIndexError { name: Some(String::from("a")), index: 5, line: 1 }));
        assert_eq!(env.get(String::from("b"), Some(Value::Number(5.0)), 1), Err(ErrorType::NotIndexableError { name: Some(String::from("b")), line: 1 }));
    }
}
