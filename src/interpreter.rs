use crate::{environment::Environment, expr::{Expr, ExprType}, token::{Value, TokenType, Literal}, error::{ErrorType, self}, stmt::{Stmt, StmtType}};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, ast: Vec<Stmt>) {
        for stmt in &ast {
            if let Err(e) = self.execute(stmt) {
                error::report(&e);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), ErrorType> {
        match &stmt.stmt_type {
            StmtType::Print { expression } => {
                println!("{}", self.evaluate(expression)?);
                Ok(())
            },
            _ => unimplemented!(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, ErrorType> {
        match &expr.expr_type {
            ExprType::Array { elements } => {
                let values: Result<Vec<Value>, _> = elements.iter().map(|x| self.evaluate(x)).collect();
                Ok(Value::Array(values?))
            },
            ExprType::Assignment { target, value } => {
                let value_eval = self.evaluate(value.as_ref())?;
                match &target.expr_type {
                    ExprType::Element { array, index } => {
                        match &array.expr_type {
                            ExprType::Array {..} => {},  // Trying to assign to literal array -> no op
                            ExprType::Variable { name } => {
                                self.environment.assign(name.clone(), Some(*index), &value_eval, target.line)?;
                            },
                            _ => return Err(ErrorType::ExpressionNotIndexable { line: array.line }),
                        }
                    },
                    ExprType::Variable { name } => {
                        self.environment.assign(name.clone(), None, &value_eval, target.line)?;
                    },
                    _ => {},
                }
                Ok(value_eval)
            },
            ExprType::Binary { left, operator, right } => {
                let left_eval = self.evaluate(left.as_ref())?;
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    TokenType::Or |
                    TokenType::And => {
                        match (&left_eval, &right_eval) {
                            (Value::Bool(left_bool), Value::Bool(right_bool)) => {
                                match operator.type_ {
                                    TokenType::Or => Ok(Value::Bool(*left_bool || *right_bool)),
                                    TokenType::And => Ok(Value::Bool(*left_bool && *right_bool)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Boolean"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },

                    TokenType::EqualEqual => Ok(Value::Bool(left_eval == right_eval)),
                    TokenType::BangEqual => Ok(Value::Bool(left_eval != right_eval)),

                    TokenType::Greater |
                    TokenType::Less |
                    TokenType::GreaterEqual |
                    TokenType::LessEqual => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => {
                                match operator.type_ {
                                    TokenType::Greater => Ok(Value::Bool(left_num > right_num)),
                                    TokenType::Less => Ok(Value::Bool(left_num < right_num)),
                                    TokenType::GreaterEqual => Ok(Value::Bool(left_num >= right_num)),
                                    TokenType::LessEqual => Ok(Value::Bool(left_num <= right_num)),
                                    _ => unreachable!(),
                                }
                            },
                            (Value::String_(left_str), Value::String_(right_str)) => {
                                match operator.type_ {
                                    TokenType::Greater => Ok(Value::Bool(left_str > right_str)),
                                    TokenType::Less => Ok(Value::Bool(left_str < right_str)),
                                    TokenType::GreaterEqual => Ok(Value::Bool(left_str >= right_str)),
                                    TokenType::LessEqual => Ok(Value::Bool(left_str <= right_str)),
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number or String"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },

                    TokenType::Plus => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => Ok(Value::Number(left_num + right_num)),
                            (Value::String_(left_str), Value::String_(right_str)) => Ok(Value::String_(format!("{}{}", left_str, right_str))),
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number or String"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    },
                    TokenType::Minus |
                    TokenType::Star |
                    TokenType::Slash => {
                        match (&left_eval, &right_eval) {
                            (Value::Number(left_num), Value::Number(right_num)) => {
                                match operator.type_ {
                                    TokenType::Minus => Ok(Value::Number(left_num - right_num)),
                                    TokenType::Star => Ok(Value::Number(left_num * right_num)),
                                    TokenType::Slash => {
                                        if *right_num == 0.0 {
                                            Err(ErrorType::DivideByZero { line: right.line })
                                        } else {
                                            Ok(Value::Number(left_num / right_num))
                                        }
                                    },
                                    _ => unreachable!(),
                                }
                            },
                            (_, _) => {
                                Err(ErrorType::BinaryTypeError {
                                    expected: String::from("Number"),
                                    got_left: left_eval.type_to_string(),
                                    got_right: right_eval.type_to_string(),
                                    line: left.line,
                                })
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            },
            ExprType::Literal { value } => {
                match value {
                    Literal::Number(x) => Ok(Value::Number(*x)),
                    Literal::String_(x) => Ok(Value::String_(x.clone())),
                    Literal::Bool(x) => Ok(Value::Bool(*x)),
                    Literal::Null => Ok(Value::Null),
                }
            }
            _ => unimplemented!(),
        }
    }
}
