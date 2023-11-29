use std::io::{Write, self};

use crate::{environment::{Environment, Pointer, self}, expr::{Expr, ExprType}, token::{TokenType, Literal}, error::{ErrorType, self}, stmt::{Stmt, StmtType}, value::{Value, BuiltinFunction}, hash_table::HashTable};

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
                error::report_errors(&[e]);
                return;
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), ErrorType> {
        match &stmt.stmt_type {
            StmtType::Block { body } => {
                self.environment.new_scope();
                for block_stmt in body {
                    self.execute(block_stmt)?;
                }
                self.environment.exit_scope();
                Ok(())
            },

            StmtType::Break => {
                Err(ErrorType::ThrownBreak { line: stmt.line })
            },

            StmtType::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            },

            StmtType::Function { name, parameters, body } => {
                self.environment.declare(name.clone(), &Value::Function {
                    parameters: parameters.clone(),
                    body: *body.clone(),
                });
                Ok(())
            },

            StmtType::If { condition, then_body, else_body } => {
                match self.evaluate(condition)? {
                    Value::Bool(condition_bool) => {
                        if condition_bool {
                            self.execute(then_body.as_ref())?;
                        } else if let Some(else_) = else_body {
                                self.execute(else_.as_ref())?;
                        }
                        Ok(())
                    },
                    _ => Err(ErrorType::IfConditionNotBoolean { line: condition.line })
                }
            },

            StmtType::Print { expression } => {
                println!("{}", self.evaluate(expression)?);
                Ok(())
            },

            StmtType::Return { expression } => {
                if let Some(expr) = expression {
                    Err(ErrorType::ThrownReturn { value: self.evaluate(expr)?, line: stmt.line })
                } else {
                    Err(ErrorType::ThrownReturn { value: Value::Null, line: stmt.line })
                }
            },

            StmtType::VarDecl { name, value } => {
                let value_eval = &self.evaluate(value)?;
                self.environment.declare(name.clone(), value_eval);
                Ok(())
            },
            
            StmtType::While { condition, body } => {
                loop {
                    let continue_ = match self.evaluate(condition)? {
                        Value::Bool(condition_bool) => {
                            condition_bool
                        },
                        _ => return Err(ErrorType::WhileConditionNotBoolean { line: stmt.line })
                    };
                    if !continue_ {
                        break;
                    }

                    let exec_result = self.execute(body.as_ref());
                    match exec_result {
                        Ok(()) => (),
                        Err(ErrorType::ThrownBreak {..}) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            },
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
                match self.construct_pointer(target, expr.line) {
                    Ok(pointer) => self.environment.update(&pointer, &value_eval, expr.line)?,
                    Err(ErrorType::ThrownLiteralAssignment { .. }) => (),  // If assign to literal, e.g. [1,2][0] = 5, do nothing
                    Err(e) => return Err(e),
                };
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
                    TokenType::Slash |
                    TokenType::Percent => {
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
                                    TokenType::Percent => Ok(Value::Number(left_num % right_num)),
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
                    },
                    _ => unreachable!(),
                }
            },

            ExprType::Call { callee, arguments } => {
                let function = self.evaluate(callee.as_ref())?;
                match function {
                    Value::Function { parameters, body } => {
                        if arguments.len() != parameters.len() {
                            return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: parameters.len(), line: expr.line })
                        }
        
                        self.environment.new_scope();
                        for i in 0..arguments.len() {  // attach arguments to scope
                            let arg_eval = self.evaluate(&arguments[i])?;
                            self.environment.declare(parameters[i].clone(), &arg_eval);
                        }
        
                        let exec_result = self.execute(&body);
                        self.environment.exit_scope();
                        
                        match exec_result {
                            Ok(()) => Ok(Value::Null),
                            Err(ErrorType::ThrownReturn { value, line: _ }) => Ok(value),
                            Err(e) => Err(e),
                        }
                    },
                    Value::BuiltinFunction(function) => {
                        match function {
                            BuiltinFunction::Append => {
                                if arguments.len() != 2 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line })
                                }

                                let target = &arguments[0];
                                let value_eval = self.evaluate(&arguments[1])?;

                                let target_eval = self.evaluate(target)?;
                                let pointer = self.construct_pointer(target, target.line)?;
                                if let Value::Array(mut array) = target_eval {
                                    array.push(value_eval);
                                    self.environment.update(&pointer, &Value::Array(array.clone()), expr.line)?;

                                    Ok(Value::Array(array))
                                } else {
                                    Err(ErrorType::ExpectedTypeError { expected: String::from("Array"), got: target_eval.type_to_string(), line: target.line })
                                }
                            },
                            BuiltinFunction::Input => {
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let prompt = self.evaluate(&arguments[0])?;
                                print!("{}", prompt);
                                io::stdout().flush().expect("Error: flush failed");

                                let mut input = String::new();
                                io::stdin().read_line(&mut input).expect("Error: something went wrong while reading input");
                                input = input.trim().to_string();

                                Ok(Value::String_(input))
                            },
                            BuiltinFunction::Remove => {
                                if arguments.len() != 2 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 2, line: expr.line })
                                }

                                let target = &arguments[0];
                                let key_eval = self.evaluate(&arguments[1])?;

                                let target_eval = self.evaluate(target)?;
                                let pointer = self.construct_pointer(target, target.line)?;
                                match target_eval {
                                    Value::Array(mut array) => {
                                        let index = environment::index_value_to_usize(&key_eval, arguments[1].line)?;

                                        if index < array.len() {
                                            array.remove(index);
                                        } else {
                                            return Err(ErrorType::OutOfBoundsIndexError { name: None, index, line: arguments[1].line });
                                        }
                                        
                                        self.environment.update(&pointer, &Value::Array(array.clone()), expr.line)?;

                                        Ok(Value::Array(array))
                                    },
                                    Value::Dictionary(mut dict) => {
                                        dict.remove(&key_eval, expr.line)?;
                                        self.environment.update(&pointer, &Value::Dictionary(dict.clone()), expr.line)?;
                                        
                                        Ok(Value::Dictionary(dict))
                                    },
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Array or Dictionary"), got: target_eval.type_to_string(), line: target.line }),
                                }
                            },
                            BuiltinFunction::Size => {
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Array(array) => Ok(Value::Number(array.len() as f64)),
                                    Value::Dictionary(dict) => Ok(Value::Number(dict.size() as f64)),
                                    Value::String_(s) => Ok(Value::Number(s.len() as f64)),
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Array, Dictionary or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            }
                            BuiltinFunction::ToNumber => {
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Bool(b) => {
                                        match b {
                                            true => Ok(Value::Number(1.0)),
                                            false => Ok(Value::Number(0.0)),
                                        }
                                    },
                                    Value::Number(..) => Ok(value),
                                    Value::String_(s) => {
                                        match s.parse::<f64>() {
                                            Ok(x) => Ok(Value::Number(x)),
                                            Err(..) => Err(ErrorType::ConvertToNumberError { line: expr.line }),
                                        }
                                    },
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                            BuiltinFunction::ToString => {
                                if arguments.len() != 1 {
                                    return Err(ErrorType::ArgParamNumberMismatch { arg_number: arguments.len(), param_number: 1, line: expr.line })
                                }

                                let value = self.evaluate(&arguments[0])?;
                                match value {
                                    Value::Bool(b) => {
                                        match b {
                                            true => Ok(Value::String_(String::from("true"))),
                                            false => Ok(Value::String_(String::from("false"))),
                                        }
                                    },
                                    Value::Number(x) => Ok(Value::String_(x.to_string())),
                                    Value::String_(..) => Ok(value),
                                    _ => Err(ErrorType::ExpectedTypeError { expected: String::from("Boolean, Number or String"), got: value.type_to_string(), line: expr.line }),
                                }
                            },
                        }
                    },
                    _ => Err(ErrorType::CannotCallName { line: callee.line })
                }
            },

            ExprType::Dictionary { elements } => {
                let mut hash_table = HashTable::new();
                for key_value in elements.iter() {
                    let key_eval = self.evaluate(&key_value.key)?;
                    let value_eval = self.evaluate(&key_value.value)?;
                    hash_table.insert(&key_eval, &value_eval, expr.line)?;
                }
                Ok(Value::Dictionary(hash_table))
            },

            ExprType::Element { array, index } => {
                let index_eval = self.evaluate(index.as_ref())?;
                match self.evaluate(array.as_ref())? {
                    Value::Array(array) => {
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;
                        if let Some(element) = array.get(index_num) {
                            Ok(element.clone())
                        } else {
                            Err(ErrorType::OutOfBoundsIndexError { name: None, index: index_num, line: expr.line })
                        }
                    },
                    Value::Dictionary(dict) => {
                        dict.get(&index_eval, expr.line).cloned()
                    },
                    Value::String_(s) => {
                        let index_num = environment::index_value_to_usize(&index_eval, index.line)?;
                        if let Some(c) = s.chars().nth(index_num) {
                            Ok(Value::String_(String::from(c)))
                        } else {
                            Err(ErrorType::OutOfBoundsIndexError { name: None, index: index_num, line: expr.line })
                        }
                    },
                    _ => Err(ErrorType::NotIndexableError { name: None, line: array.line })
                    // ExprType::Array { elements } => {
                    //     let index_num = environment::index_value_to_usize(&index_eval, expr.line)?;
                    //     if let Some(element_expr) = elements.get(index_num) {
                    //         self.evaluate(element_expr)
                    //     } else {
                    //         Err(ErrorType::OutOfBoundsIndexError { name: None, index: index_num, line: expr.line })
                    //     }
                    // },
                    // ExprType::Dictionary { elements } => {
                    //     for key_value_expr in elements.iter().rev() {  // Go in reverse to emulate normal `replacing` behaviour.
                    //         let key_eval = self.evaluate(&key_value_expr.key)?;
                    //         if index_eval == key_eval {
                    //             return self.evaluate(&key_value_expr.value);
                    //         }
                    //     }
                    //     Err(ErrorType::KeyError { key: index_eval, line: expr.line })
                    // },
                    // ExprType::Variable { name } => {
                    //     self.environment.get(name.clone(), Some(index_eval), expr.line)
                    // },
                    // _ => Err(ErrorType::NotIndexableError { name: None, line: array.line }),
                }
            },

            ExprType::Grouping { expression } => {
                self.evaluate(expression.as_ref())
            },

            ExprType::Literal { value } => {
                match value {
                    Literal::Number(x) => Ok(Value::Number(*x)),
                    Literal::String_(x) => Ok(Value::String_(x.clone())),
                    Literal::Bool(x) => Ok(Value::Bool(*x)),
                    Literal::Null => Ok(Value::Null),
                }
            },

            ExprType::Unary { operator, right } => {
                let right_eval = self.evaluate(right.as_ref())?;

                match operator.type_ {
                    TokenType::Bang => {
                        match right_eval {
                            Value::Bool(right_bool) => Ok(Value::Bool(!right_bool)),
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Boolean"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    TokenType::Minus => {
                        match right_eval {
                            Value::Number(right_num) => Ok(Value::Number(-right_num)),
                            _ => Err(ErrorType::ExpectedTypeError {
                                expected: String::from("Number"),
                                got: right_eval.type_to_string(),
                                line: right.line,
                            })
                        }
                    },
                    _ => unreachable!(),
                }
            },

            ExprType::Variable { name } => {
                self.environment.get(name.clone(), expr.line)
            },
        }
    }

    fn construct_pointer(&mut self, element: &Expr, line: usize) -> Result<Pointer, ErrorType> {
        match &element.expr_type {
            ExprType::Element { array, index } => {
                let Pointer {name, indices} = self.construct_pointer(array.as_ref(), line)?;
                let mut indices_copy = indices;
                indices_copy.push(self.evaluate(index.as_ref())?);
                Ok(Pointer {name, indices: indices_copy})
            },
            ExprType::Variable { name } => {
                Ok(Pointer {name: name.clone(), indices: Vec::new()})
            },
            ExprType::Array {..} | ExprType::Call {..} | ExprType::Dictionary {..} => Err(ErrorType::ThrownLiteralAssignment { line }),
            _ => Err(ErrorType::InvalidAssignmentTarget { line }),
        }
    }
}
