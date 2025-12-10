use super::ast::{Expr, Program};
use super::abstractions;
use super::ast::{BinaryOp, Statement};

use std::{cell::RefCell, collections::HashMap, rc::Rc};
use anyhow::{Ok, Result, bail};

#[derive(Debug, Clone)]
pub enum EvaluationValue {
    Literal(f64),
    // basically, a closure
    Closer(Rc<Abstraction>),
    // The unit value, for statements that don't produce a visible result (like bindings).
    Unit,
    Recursion(Box<Expr>),

    // any lamda receiving the signal "HALT" must not execute
    // eg. (λprint.(λinput. (λif) 0 0))
    // (λprint.(λinput. 'HALT'))
    // (λprint.'HALT')
    // 'HALT'
    // HALT can be stored into a variable.
    HALT,
}

type Environment = Rc<RefCell<Scope>>;

#[derive(Debug, Clone)]

pub struct Abstraction {
    param: String,
    body: Box<Expr>,
    env: Environment,
}

#[derive(Debug)]

pub struct Scope {
    bindings: HashMap<String, EvaluationValue>,
    parent: Option<Environment>,
}

impl Scope {
    pub fn global() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Scope {
            bindings: HashMap::new(),
            parent: None,
        }))
    }

    pub fn inner(parent: Rc<RefCell<Scope>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Scope {
            bindings: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn set(&mut self, name: String, value: EvaluationValue) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<EvaluationValue> {
        match self.bindings.get(name) {
            Some(val) => Some(val.clone()),
            None => match &self.parent {
                Some(parent) => parent.borrow().get(name),
                None => None,
            },
        }
    }
}

pub struct Interpreter {
    env: Rc<RefCell<Scope>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Scope::global(),
        }
    }
    pub fn evaluate_program(&mut self, program: &Program) -> Result<Vec<EvaluationValue>> {
        let mut results = Vec::new();
        for statement in &program.statements {
            let result = self.evaluate_statement(statement)?;
            results.push(result);
        }
        Ok(results)
    }

    fn evaluate_statement(&mut self, statement: &Statement) -> Result<EvaluationValue> {
        match statement {
            Statement::Binding { name, value } => {
                let evaluated_value = self.evaluate_expr(value, Rc::clone(&self.env))?;
                self.env
                    .borrow_mut()
                    .set(name.clone(), evaluated_value.clone());
                Ok(evaluated_value)
            }
            Statement::ExpressionStmt(expr) => self.evaluate_expr(expr, Rc::clone(&self.env)),
            Statement::Comment(_) | Statement::Eof => Ok(EvaluationValue::Unit),
        }
    }
    // Evaluate expression in the given environment
    fn evaluate_expr(&mut self, expr: &Expr, env: Environment) -> Result<EvaluationValue> {
        match expr {
            // Expr::Literal(literal) if *literal == 0. => Ok(EvaluationValue::Literal(*literal)),
            Expr::Literal(literal) => Ok(EvaluationValue::Literal(*literal)),

            Expr::Identifier(name) => match env.borrow().get(name) {
                Some(val) => Ok(val),
                None => bail!("Unbound binding: {}", name),
            },
            Expr::BinaryOperation { op, lhs, rhs } => self.evaluate_binary(op, lhs, rhs, env),
            Expr::Abstraction { param, body } => {
                Ok(EvaluationValue::Closer(Rc::new(Abstraction {
                    param: param.clone(),
                    body: body.clone(),
                    env: Rc::clone(&env),
                })))
            }
            Expr::Application { func, arg } => {
                self.evaluate_appliation(func, arg, Rc::clone(&env), false)
            }
            Expr::ApplicationIf { func, arg1, arg2 } => {
                self.evaluate_appliationif(func, arg1, arg2, Rc::clone(&env))
            }
            Expr::Recursion(args) => Ok(EvaluationValue::Recursion(args.clone())),
        }
    }

    fn evaluate_appliationif(
        &mut self,
        func: &Box<Expr>,
        arg1: &Box<Expr>,
        arg2: &Box<Expr>,
        env: Environment,
    ) -> Result<EvaluationValue> {
        let evaluated_func_value = self.evaluate_appliation(&func, arg1, Rc::clone(&env), false)?;
        let arg2 = self.evaluate_expr(&arg2, Rc::clone(&env))?;
        match evaluated_func_value {
            EvaluationValue::Literal(1.) => Ok(arg2),
            EvaluationValue::Literal(_) => Ok(EvaluationValue::HALT),
            _ => bail!("λif only takes numeric value"),
        }
    }

    fn evaluate_binary(
        &mut self,
        op: &BinaryOp,
        lhs: &Box<Expr>,
        rhs: &Box<Expr>,
        env: Environment,
    ) -> Result<EvaluationValue> {
        let lhs_result = self.evaluate_expr(lhs, Rc::clone(&env))?;
        let rhs_result = self.evaluate_expr(rhs, Rc::clone(&env))?;
        let (EvaluationValue::Literal(l), EvaluationValue::Literal(r)) = (lhs_result, rhs_result)
        else {
            bail!("Expected numeric literal for binary operations")
        };
        let result = match op {
            BinaryOp::Add => l + r,
            BinaryOp::Sub => l - r,
            BinaryOp::Mul => l * r,
            BinaryOp::Div => l / r,
            BinaryOp::BitAnd => ((l as u64) & (r as u64)) as f64,
            BinaryOp::BitOr => ((l as u64) | (r as u64)) as f64,
        };
        Ok(EvaluationValue::Literal(result))
    }

    fn evaluate_appliation(
        &mut self,
        func: &Box<Expr>,
        arg: &Box<Expr>,
        env: Environment,
        recursion_context: bool,
    ) -> Result<EvaluationValue> {
        // can be func, just want make them equal in length, ahh equal length 😭
        let evaluated_fun_value = self.evaluate_expr(&func, Rc::clone(&env))?;
        let evaluated_arg_value = self.evaluate_expr(&arg, Rc::clone(&env))?;

        if recursion_context {
            if let EvaluationValue::Literal(0.) = evaluated_arg_value {
                return Ok(EvaluationValue::HALT);
            }
        }
        match evaluated_fun_value {
            EvaluationValue::Closer(abstraction) => {
                // Scope of abstraciton diffrs from the global context/scope, creating new scope/environment;
                // Where, current env is a captured env.
                // if the abbtraction were to be applied from another abstraction, then it no longer can access
                // gloabl abstraction so, putting the previously captured environment.
                let new_env = Scope::inner(Rc::clone(&abstraction.env));

                // binding parameters.
                new_env
                    .borrow_mut()
                    .set(abstraction.param.clone(), evaluated_arg_value);

                let mut func_result = self.evaluate_expr(&abstraction.body, Rc::clone(&new_env))?;

                let mut recursion_args = None;
                if let EvaluationValue::Recursion(rec_args) = func_result {
                    func_result = self.evaluate_expr(&rec_args, Rc::clone(&new_env))?;
                    // Only a single depth and a valid literal or halt signal is allowed.
                    match func_result {
                        EvaluationValue::Literal(_) | EvaluationValue::HALT => (),
                        _ => bail!("Recursion(𝑓) only takes numeric value."),
                    }
                    // Store for later use so, it can ran after abstraction has been evaluated.
                    recursion_args = Some(rec_args);
                }

                if matches!(func_result, EvaluationValue::HALT) {
                    return Ok(EvaluationValue::HALT);
                }

                // Kinda bit of redundancy
                let func_evalution_result = match abstraction.param.as_str() {
                    "ascii" => match func_result {
                        EvaluationValue::Literal(ascii) if ascii >= 255.0 => bail!(
                            "λascii only takes ASCII values in decimal form, ranging from 0 to 255."
                        ),
                        EvaluationValue::Literal(ascii) => {
                            abstractions::abstraction_ascii(ascii as u8)
                        }
                        _ => bail!(
                            "λascii only takes ASCII values in decimal form, ranging from 0 to 255.",
                        ),
                    },
                    "input" => match func_result {
                        EvaluationValue::Literal(0.) => abstractions::abstraction_input_char(),

                        EvaluationValue::Literal(1.) => abstractions::abstraction_input_numeric(),
                        EvaluationValue::Literal(_) => {
                            bail!("λinput only takes numeric value either, 0, or 1.")
                        }
                        _ => bail!("λinput only takes numeric value either, 0, or 1.",),
                    },
                    "time" => abstractions::abstraction_time(),
                    "print" => match func_result {
                        EvaluationValue::Literal(numeric_value) => {
                            abstractions::abstraction_print(numeric_value)
                        }
                        _ => bail!("λraw only takes numeric value."),
                    },

                    "sleep" => match func_result {
                        EvaluationValue::Literal(numeric_value) => {
                            abstractions::abstraction_sleep(numeric_value)
                        }
                        _ => bail!("λraw only takes numeric value."),
                    },
                    _ => Ok(func_result),
                };

                if let Some(rec_args) = recursion_args {
                    return self.evaluate_appliation(&func, &rec_args, Rc::clone(&new_env), true);
                }
                func_evalution_result
            }
            EvaluationValue::Literal(literal) => Ok(EvaluationValue::Literal(literal)),
            EvaluationValue::Unit => Ok(EvaluationValue::Unit),
            EvaluationValue::HALT => Ok(EvaluationValue::HALT),
            _ => bail!("Unexpected evaluation value!"),
        }
    }
}
