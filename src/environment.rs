use std::collections::HashMap;

use crate::{token::Value, error::{ErrorType, self}};

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
        if self.scopes.len() == 0 {
            panic!("Exited out of global scope.");
        }
    }

    pub fn declare(&mut self, name: String, value: Value) {
        if let Some(last_scope) = self.scopes.last_mut() {
            last_scope.insert(name, value);
        } else {
            panic!("No scopes to declare to.");
        }
    }

    pub fn get(&self, name: String, line: usize) -> Result<Value, ErrorType> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&name) {
                return Ok(value.clone());
            }
        }
        Err(ErrorType::NameError { name, line })
    }

    pub fn assign(&mut self, name: String, value: Value, line: usize) -> Result<(), ErrorType> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
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
        env.declare(String::from("a"), Value::Number(5.0));
        env.declare(String::from("b"), Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))]));
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(5.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Array(vec![Value::Bool(true), Value::String_(String::from("hello world!"))])));

        let _ = env.assign(String::from("b"), Value::String_(String::from("abc")), 1);
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
        env.declare(String::from("a"), Value::Number(1.0));
        env.declare(String::from("b"), Value::Number(2.0));

        env.new_scope();
        let _ = env.assign(String::from("a"), Value::Number(10.0), 1);
        env.declare(String::from("b"), Value::Number(20.0));
        assert_eq!(env.get(String::from("a"), 1), Ok(Value::Number(10.0)));
        assert_eq!(env.get(String::from("b"), 1), Ok(Value::Number(20.0)));

        env.new_scope();
        let _ = env.assign(String::from("b"), Value::Number(30.0), 1);
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
        assert_eq!(env.assign(String::from("b"), Value::Null, 1), Err(ErrorType::NameError { name: String::from("b"), line: 1 }));
    }
}
