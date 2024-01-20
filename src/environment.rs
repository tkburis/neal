use std::collections::HashMap;

use crate::{value::{Value, BuiltinFunction}, error::ErrorType};

/// Allows the updating of elements in multi-dimensional arrays and dictionaries.
#[derive(Debug)]
pub struct Pointer {
    pub name: String,  // The name of the 'base' array or dictionary.
    pub indices: Vec<Value>,  // The sequence of indices needed to access the element.
}

/// Stores variables and functions.
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,  // The 'linked list' of variable scopes. Each scope contains a hash map of name-value pairs.
}

impl Environment {
    /// Initialises a new instance of `Environment`.
    pub fn new() -> Self {
        Self {
            // Initialises the built-in functions in the base scope.
            scopes: vec![HashMap::from([
                (String::from("append"), Value::BuiltinFunction(BuiltinFunction::Append)),
                (String::from("input"), Value::BuiltinFunction(BuiltinFunction::Input)),
                (String::from("remove"), Value::BuiltinFunction(BuiltinFunction::Remove)),
                (String::from("size"), Value::BuiltinFunction(BuiltinFunction::Size)),
                (String::from("sort"), Value::BuiltinFunction(BuiltinFunction::Sort)),
                (String::from("to_number"), Value::BuiltinFunction(BuiltinFunction::ToNumber)),
                (String::from("to_string"), Value::BuiltinFunction(BuiltinFunction::ToString)),
            ])],
        }
    }

    /// Creates and enters a new scope.
    pub fn new_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exits and removes the outermost scope.
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
        if self.scopes.is_empty() {
            panic!("Exited out of base scope.");
        }
    }

    /// Declares a name-value pair in the current scope.
    pub fn declare(&mut self, name: String, value: &Value) {
        if let Some(last_scope) = self.scopes.last_mut() {
            // If there is at least one scope, insert the name-value pair into the outermost scope.
            last_scope.insert(name, value.clone());
        } else {
            // Should be unreachable.
            panic!("No scopes to declare to.");
        }
    }

    /// Returns the value associated with `name`. As there could be multiple values associated with `name`
    /// across all the scopes, return the one in the outermost scope.
    pub fn get(&self, name: String, line: usize) -> Result<Value, ErrorType> {
        for scope in self.scopes.iter().rev() {
            // Iterate from the outermost scope.
            if let Some(object) = scope.get(&name) {
                // If there is a value associated with `name`, return the value immediately.
                return Ok(object.clone());
            }
        }
        // We have iterated through all the scopes and no value have been found to be associated with `name`.
        // So raise a `NameError`, giving the `name` in question to be as detailed as possible.
        Err(ErrorType::NameError { name, line })
    }

    /// Updates the value associated with the pointer. Again, update the one in the outermost scope only.
    pub fn update(&mut self, pointer: &Pointer, value: &Value, line: usize) -> Result<(), ErrorType> {
        for scope in self.scopes.iter_mut().rev() {
            // Iterate from the outermost scope.
            if let Some(object) = scope.get_mut(&pointer.name) {
                // If there is a value associated with `pointer.name`...
                if !pointer.indices.is_empty() {
                    // If indices were provided...

                    // This is the array/dictionary associated with `pointer.name`.
                    let mut current_element = object;

                    // For each index in `pointer.indices` except the last, replace `current_element` with `current_element[index]`.
                    for i in pointer.indices.iter().take(pointer.indices.len() - 1) {
                        match current_element {
                            Value::Array(array) => {
                                // If `current_element` is an array, we have to convert the index into `usize` and make sure
                                // it is not out-of-bounds.
                                let idx = index_value_to_usize(i, line)?;
                                if let Some(el) = array.get_mut(idx) {
                                    current_element = el;
                                } else {
                                    // If the index provided is out-of-bounds, raise an `OutOfBoundsIndexError`.
                                    return Err(ErrorType::OutOfBoundsIndexError { index: idx, line });
                                }
                            },
                            Value::Dictionary(dict) => {
                                // If `current_element` is a dictionary, we can let `HashTable` get `current_element[index]`.
                                current_element = dict.get_mut(i, line)?;
                            },
                            // If it is any other variant of `Value`, then we cannot index it.
                            // Note: strings can only be indexed with the last index, so it is not included here.
                            _ => return Err(ErrorType::NotIndexableError { line }),
                        }
                    }

                    // Note that the last index is separated so that:
                    // 1. Dictionaries can insert key-value pairs with the last key if it does not exist already.
                    //    For example, `a[1][5] = 1` inserts `5` as a key if it does not exist already (`a[1]` is a dictionary).
                    // 2. For strings, you have to do it this way to allow mutations like `a[2][1] = 'h'`.
                    let last_index = pointer.indices.last().unwrap();
                    match current_element {
                        Value::Array(array) => {
                            // As above.
                            let idx = index_value_to_usize(last_index, line)?;
                            if let Some(el) = array.get_mut(idx) {
                                current_element = el;
                            } else {
                                // If the index provided is out-of-bounds or similar...
                                return Err(ErrorType::OutOfBoundsIndexError { index: idx, line });
                            }
                            *current_element = value.clone();
                        },
                        Value::Dictionary(dict) => {
                            // `HashTable` inserts key-value pairs if the key does not exist already and updates them otherwise.
                            dict.insert(last_index, value, line)?;
                        },
                        Value::String_(s) => {
                            // Convert the index value into a `usize`.
                            let idx = index_value_to_usize(last_index, line)?;

                            // Make sure it is not out-of-bounds.
                            if s.get(idx..idx+1).is_none() {
                                return Err(ErrorType::OutOfBoundsIndexError { index: idx, line });
                            }

                            if let Value::String_(c) = value {
                                // If `value` is a string, replace `current_element[index]` with `value`.
                                s.replace_range(idx..idx+1, c);
                            } else {
                                // Otherwise, it cannot be inserted into a string.
                                return Err(ErrorType::InsertNonStringIntoStringError { line });
                            }
                        },
                        // Any other variant of `Value` cannot be indexed.
                        _ => return Err(ErrorType::NotIndexableError { line }),
                    }

                    return Ok(());

                } else {
                    // If no indices were provided, simply replace the value associated with `pointer.name` with `value`.
                    // Note: `insert()` will update the key-value pair if the key exists already.
                    scope.insert(pointer.name.clone(), value.clone());
                    return Ok(());
                }
            }
        }
        // We have iterated through all the scopes and no value have been found to be associated with `name`.
        // So raise a `NameError`, giving the `name` in question to be as detailed as possible.
        Err(ErrorType::NameError { name: pointer.name.clone(), line })
    }
}

/// Converts a variant of `Value` into a usize. If it cannot, raises an appropriate error.
pub fn index_value_to_usize(index: &Value, line: usize) -> Result<usize, ErrorType> {
    match index {
        Value::Number(index_num) => {
            // If `index` is a `Number` variant...

            // and the number is non-negative and an integer...
            if *index_num >= 0.0 && index_num.fract() == 0.0 {
                // ...convert it into a `usize`.
                Ok(*index_num as usize)
            } else {
                // If it is not non-negative or it is not an integer, then raise an error as it cannot be used as an index.
                Err(ErrorType::NonNaturalIndexError { got: index.clone(), line })
            }
        },
        // If it is not a `Number` variant, then it cannot be used as an index, so raise an error.
        _ => Err(ErrorType::NonNumberIndexError { got: index.type_to_string(), line })
    }
}

#[cfg(test)]
mod tests {
    use crate::{value::Value, error::ErrorType, environment::Pointer};

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

        let _ = env.update(&Pointer { name: String::from("b"), indices: vec![] }, &Value::String_(String::from("abc")), 1);
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
        let _ = env.update(&Pointer { name: String::from("a"), indices: vec![] }, &Value::Number(10.0), 1);
        env.declare(String::from("b"), &Value::Number(20.0));
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(20.0)));

        env.new_scope();
        let _ = env.update(&Pointer { name: String::from("b"), indices: vec![] }, &Value::Number(30.0), 1);
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
        assert_eq!(env.update(&Pointer { name: String::from("b"), indices: vec![] }, &Value::Null, 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }

    #[test]
    fn declare_twice() {
        let mut env = Environment::new();
        env.declare(String::from("b"), &Value::Number(123.0));
        env.declare(String::from("b"), &Value::Number(55.0));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(55.0)));
    }
}
