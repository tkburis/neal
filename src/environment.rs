use std::collections::HashMap;

use crate::{value::{Value, BuiltinFunction}, error::ErrorType};

#[derive(Debug)]
pub struct AssignmentPointer {
    pub name: String,
    pub indices: Vec<Value>,
}

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::from([
                (String::from("append"), Value::BuiltinFunction(BuiltinFunction::Append)),
                (String::from("input"), Value::BuiltinFunction(BuiltinFunction::Input)),
                (String::from("remove"), Value::BuiltinFunction(BuiltinFunction::Remove)),
                (String::from("size"), Value::BuiltinFunction(BuiltinFunction::Size)),
                (String::from("to_number"), Value::BuiltinFunction(BuiltinFunction::ToNumber)),
                (String::from("to_string"), Value::BuiltinFunction(BuiltinFunction::ToString)),
            ])],
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

    pub fn get(&self, name: String, line: usize) -> Result<Value, ErrorType> {
        for scope in self.scopes.iter().rev() {
            if let Some(object) = scope.get(&name) {
                // If there is a value associated with `name`...
                return Ok(object.clone());
            }
        }
        Err(ErrorType::NameError { name, line })
    }

    pub fn update(&mut self, pointer: &AssignmentPointer, value: &Value, line: usize) -> Result<(), ErrorType> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(object) = scope.get_mut(&pointer.name) {
                // If there is a value associated with `name`...
                if !pointer.indices.is_empty() {
                    // If indeces were provided...
                    let mut current_element = object;
                    for i in pointer.indices.iter().take(pointer.indices.len() - 1) {  // all but last element
                        match current_element {
                            Value::Array(array) => {
                                let idx = index_value_to_usize(i, line)?;
                                if let Some(el) = array.get_mut(idx) {
                                    current_element = el;
                                } else {
                                    // If the index provided is out-of-bounds or similar...
                                    return Err(ErrorType::OutOfBoundsIndexError { name: None, index: idx, line })
                                }
                            },
                            Value::Dictionary(dict) => {
                                current_element = dict.get_mut(i, line)?;
                            },
                            _ => return Err(ErrorType::NotIndexableError { name: None, line }),
                        }
                    }

                    // Why? So that a[1][5] = 1 inserts `5` as a key if it does not exist already.
                    // For strings, you have to do it this way to allow a[1] = 'h'.
                    let last_index = pointer.indices.last().unwrap();
                    match current_element {
                        Value::Array(array) => {
                            let idx = index_value_to_usize(last_index, line)?;
                            if let Some(el) = array.get_mut(idx) {
                                current_element = el;
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                return Err(ErrorType::OutOfBoundsIndexError { name: None, index: idx, line })
                            }
                            *current_element = value.clone();
                        },
                        Value::Dictionary(dict) => {
                            dict.insert(last_index, value, line)?;
                        },
                        Value::String_(s) => {
                            let idx = index_value_to_usize(last_index, line)?;
                            if s.get(idx..idx+1).is_none() {
                                return Err(ErrorType::OutOfBoundsIndexError { name: None, index: idx, line });
                            }
                            if let Value::String_(c) = value {
                                s.replace_range(idx..idx+1, c);
                            } else {
                                return Err(ErrorType::InsertNonStringIntoStringError { line });
                            }
                        },
                        _ => return Err(ErrorType::NotIndexableError { name: None, line }),
                    }

                    return Ok(());
                } else {
                    // If no indeces were provided...
                    scope.insert(pointer.name.clone(), value.clone());
                    return Ok(());
                }
            }
        }
        Err(ErrorType::NameError { name: pointer.name.clone(), line })
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
    use crate::{value::Value, error::ErrorType, environment::AssignmentPointer};

    use super::Environment;

    #[test]
    fn one_scope() {
        //  var a = 5
        //  var b = [true, "hello world!"]
        //  b = "abc"
        let mut env = Environment::new();
        env.declare(String::from("a"), &Value::Number(5.0));
        env.declare(String::from("b"), &Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))]));
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))])));

        let _ = env.update(&AssignmentPointer { name: String::from("b"), indices: vec![] }, &Value::String_(String::from("abc")), 1);
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::String_(String::from("abc"))));
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
        let _ = env.update(&AssignmentPointer { name: String::from("a"), indices: vec![] }, &Value::Number(10.0), 1);
        env.declare(String::from("b"), &Value::Number(20.0));
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(20.0)));

        env.new_scope();
        let _ = env.update(&AssignmentPointer { name: String::from("b"), indices: vec![] }, &Value::Number(30.0), 1);
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(30.0)));

        env.exit_scope();
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(30.0)));

        env.exit_scope();
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(2.0)));
    }

    #[test]
    fn name_error_get() {
        let env = Environment::new();
        assert_eq!(env.get(String::from("b"), 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }

    #[test]
    fn name_error_assign() {
        let mut env = Environment::new();
        assert_eq!(env.update(&AssignmentPointer { name: String::from("b"), indices: vec![] }, &Value::Null, 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }

    #[test]
    fn declare_twice() {
        let mut env = Environment::new();
        env.declare(String::from("b"), &Value::Number(123.0));
        env.declare(String::from("b"), &Value::Number(55.0));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(55.0)));
    }
}
