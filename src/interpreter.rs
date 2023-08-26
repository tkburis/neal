use crate::{environment::Environment, expr::Expr, token::Value, error::ErrorType};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, ErrorType> {
        match expr {
            Expr::Array { elements } => {
                let values: Vec<Value> = elements.iter().map(|x| self.evaluate(x)).collect()?;
                Ok(Value::Array(values))
            },
            Expr::Assignment { target, value } => {
                let value_eval = self.evaluate(value)?;
                match **target {
                    Expr::Element { array, index } => {
                        match *array {
                            Expr::Array { elements } => (),
                            Expr::Variable { name } => {
                                self.environment.assign(name, Some(index))
                            }
                        }
                    },
                    Expr::Variable { name } => {

                    },
                    _ => {},
                }
                self.environment.assign(name, index, value, line)
            },
            Expr::Binary { left, operator, right } => {},
            Expr::Call { callee, arguments } => {},

        }
    }
}
