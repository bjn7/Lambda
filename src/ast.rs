use super::lexer::TokenKind;
use super::throw_syntax_error;
use crate::lexer::Operator;

use anyhow::{Result, bail};
use std::collections::HashMap;
#[derive(Debug, Clone)]

pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    // name = λparam. body
    // PI = 3.14
    Binding {
        name: String,
        value: Expr,
    },

    // A standalone expression, e.g.:
    // (function_x) 2 - (function_t) 3
    //  (λparam. body) 2
    ExpressionStmt(Expr),
    Eof,
    // well, I thought i could do something with comment but ig, i don't really need it huh?
    #[allow(unused)]
    Comment(String),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Abstraction {
        // lambda abstraction: λx. body
        param: String,
        body: Box<Expr>,
    },
    Literal(f64),
    Recursion(Box<Expr>),

    // wasn't planning to add this, but ig it is kinda required

    // This is a special type of application.
    // Ths application, returns arg2 if arg1 is 1.
    // else, the lamda is halted.
    // arg2 is captured from surrounding.
    // The function still receives single params which is arg1

    // syntax (λidentifier.expr) expr expr
    // eg. (λprint.print) 1 10
    // here, \n is printed since, 1 is arg1.

    // Halt signal is properly descibed in interpreter > EvaluationValue
    #[allow(unused)]
    ApplicationIf {
        func: Box<Expr>,
        arg1: Box<Expr>,
        arg2: Box<Expr>,
    },

    Application {
        // Function application
        func: Box<Expr>,
        arg: Box<Expr>,
    },
    BinaryOperation {
        // binary arithmetic
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitOr,
}

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Sum,     // + -
    Product, // * /
    Bitwise, // & |
    #[allow(unused)]
    Call, // function application (f x)
}

pub struct Parser {
    tokens: Vec<TokenKind>,
    bindings: HashMap<String, Expr>,
}

impl Parser {
    pub fn parse_program(mut tokens: Vec<TokenKind>) -> Result<Program> {
        let mut statements: Vec<Statement> = Vec::new();
        tokens.reverse();
        let mut this = Self {
            tokens,
            bindings: HashMap::new(),
        };
        while !this.tokens.is_empty() {
            statements.push(this.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.look_ahead() {
            Some(TokenKind::Identifier(_)) => {
                // See if this is a binding: "name = ..."
                if let Some(TokenKind::Operator(Operator::Equal)) =
                    self.tokens.get(self.tokens.len() - 2)
                {
                    self.parse_binding()
                } else {
                    Ok(Statement::ExpressionStmt(
                        self.parse_expression(Precedence::Lowest)?,
                    ))
                }
            }

            Some(TokenKind::Lamda)
            | Some(TokenKind::Literal(_))
            | Some(TokenKind::Operator(Operator::LeftParen)) => Ok(Statement::ExpressionStmt(
                self.parse_expression(Precedence::Lowest)?,
            )),
            Some(TokenKind::Comment(_)) => self.parse_comment(),
            Some(TokenKind::Eof) => {
                self.tokens.pop();
                Ok(Statement::Eof)
            }
            toke => bail!("Unexpected token at start of statement {:?}", toke),
        }
    }
    fn parse_comment(&mut self) -> Result<Statement> {
        if let Some(TokenKind::Comment(comment)) = self.consume() {
            Ok(Statement::Comment(comment))
        } else {
            throw_syntax_error!("comment", "Unexpected token")
        }
    }

    fn parse_binding(&mut self) -> Result<Statement> {
        // Expect identifier
        let name = if let Some(TokenKind::Identifier(name)) = self.consume() {
            name
        } else {
            bail!("Expected identifier in binding")
        };

        // Expect '='
        self.consume_expect(TokenKind::Operator(Operator::Equal));
        // Parse the value
        let value = self.parse_expression(Precedence::Lowest)?;
        self.bindings.insert(name.clone(), value.clone());
        Ok(Statement::Binding { name, value })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_prefix();
        while let Some(token) = self.look_ahead() {
            let next_prec = self.get_precedence(token);
            if next_prec <= precedence {
                break;
            }
            left = self.parse_infix(left?);
        }
        left
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.consume() {
            Some(TokenKind::Identifier(name)) => Ok(Expr::Identifier(name.clone())),
            Some(TokenKind::Literal(number)) => Ok(Expr::Literal(number.clone())),
            Some(TokenKind::Lamda) => self.parse_abstraction(),
            Some(TokenKind::Recursion) => self.parse_recursion(),
            Some(TokenKind::Operator(Operator::LeftParen)) => {
                let expr = self.parse_expression(Precedence::Call)?;
                self.consume_expect(TokenKind::Operator(Operator::RightParen));
                Ok(Expr::Application {
                    func: Box::new(expr),
                    arg: Box::new(self.parse_expression(Precedence::Lowest)?),
                })

                // Removed applicationIf support.

                // let expr = self.parse_expression(Precedence::Call)?;

                // let arg1 = self.parse_expression(Precedence::Lowest)?;

                // if self.look_expect(TokenKind::Operator(Operator::RightParen)) {
                //     self.consume_expect(TokenKind::Operator(Operator::RightParen));
                //     Ok(Expr::Application {
                //         func: Box::new(expr),
                //         arg: Box::new(arg1),
                //     })
                // } else {
                //     let arg2 = self.parse_expression(Precedence::Lowest)?;
                //     self.consume_expect(TokenKind::Operator(Operator::RightParen));
                //     Ok(Expr::ApplicationIf {
                //         func: Box::new(expr),
                //         arg1: Box::new(arg1),
                //         arg2: Box::new(arg2),
                //     })
                // }
            }
            // +------------------------------------------------------------------------------------------------------------------------------------------+
            // |The original plan was to incorporate both applied functions and the BODMAS/PENDAS rule                                                    |
            // |with   the   same parentheses.  The   approach  was   that  if the   current  token is                                                    |
            // |a left parenthesis, the expression inside would be parsed, and if it’s an abstraction,                                                    |
            // |it   would  proceed accordingly.   Next,    the   following  tokens  would   be parsed                                                    |
            // |as   statements.  If the  tokens   indicated  bindings,  the  function   would  check if                                                    |
            // |the  binding is for  an   abstraction.  If   confirmed  as an   applied  function, the                                                    |
            // |following   tokens  would be treated  as   statements,  and  non-expressions  would be                                                    |
            // |filtered out. This was done  to differentiate between a  binding immediately following                                                    |
            // |an application, which is unexpected since bindings aren’t meant to be expressions.                                                        |
            // |                                                                                                                                          |
            // |Problem:                                                                                                                                  |
            // |Consider the code:                                                                                                                        |
            // |a      = λx.λy.x*y                                                                                                                        |
            // |value = ((a) 2) 10                                                                                                                        |
            // |                                                                                                                                          |
            // |This code is perfectly valid. The number 2 is passed to a, and the resulting abstraction is then applied to 10.                           |
            // |Therefore, the expected output should be 2 * 10. However, the syntax tree fails to produce this result.                                   |
            // |                                                                                                                                          |
            // |Solution:                                                                                                                                 |
            // |Parentheses should no longer coexist with both applied functions and PENDAS. Instead, parentheses should only represent applied functions.|
            // |Inside the parentheses, there must be an applied abstraction.                                                                             |
            // +------------------------------------------------------------------------------------------------------------------------------------------+
            //

            // Reason for the solution: We could determine if it is really an application by semi-evaluation and resolving.
            // Otherwise, there must be numbers/variables using the PENDAS/BODMAS rule.
            // This approach was infeasible and unnecessarily complex.

            // Some(TokenKind::Operator(Operator::LeftParen)) => {
            //     let expr = self.parse_expression(Precedence::Lowest);
            //     self.expect(TokenKind::Operator(Operator::RightParen));
            //     match expr {
            //         Expr::Abstraction { param, body } => {
            //             if self.look_ahead().is_none() {
            //                 // Nothing after applying abstraction, so no expressions.
            //                 throw_syntax_error!("Expression", "EOF");
            //             }
            //             // parsing as a statement, caz there could be bindings, comments, or expression, after applied function.
            //             if let Statement::ExpressionStmt(expr) = self.parse_statement() {
            //                 Expr::Application {
            //                     func: Box::new(Expr::Abstraction { param, body }),
            //                     arg: Box::new(expr),
            //                 }
            //             } else {
            //                 throw_syntax_error!("expression", "statement");
            //             }
            //         }
            //         Expr::Identifier(identifier) => {
            //             if self.is_abstraction_identifier(&identifier) {
            //                 // parsing as a statement, caz there could be bindings, comments, or expression, after applied function.
            //                 if let Statement::ExpressionStmt(expr) = self.parse_statement() {
            //                     Expr::Application {
            //                         func: Box::new(Expr::Identifier(identifier)),
            //                         arg: Box::new(expr),
            //                     }
            //                 } else {
            //                     throw_syntax_error!("expression", "statement");
            //                 }
            //             } else {
            //                 self.parse_expression(Precedence::Lowest)
            //             }
            //         }
            //         expr => expr,
            //     }
            // }
            e => bail!("Unexpected token in prefix position {:?}", e),
        }
    }

    // fn expect_identifier_to_be(&self, Expr::Identifier(identifier): Expr, expr: Expr) {
    // fn is_abstraction_identifier(&self, identifier: &String) -> bool {
    //     match self.bindings.get(identifier) {
    //         Some(&Expr::Abstraction { .. }) => true,
    //         Some(&Expr::Identifier(ref identifier)) => self.is_abstraction_identifier(identifier),
    //         Some(_) => false,
    //         None => false,
    //     }
    // }

    fn consume_expect(&mut self, expected: TokenKind) {
        if let Some(token) = self.consume() {
            if token != expected {
                throw_syntax_error!(format!("{:?}", token), format!("{:?}", expected));
            }
        } else {
            throw_syntax_error!(format!("{:?}", expected), "None");
        }
    }
    
    #[allow(unused)]
    fn look_expect(&self, expected: TokenKind) -> bool {
        match self.look_ahead() {
            Some(token) if *token == expected => true,
            Some(_) => false,
            None => throw_syntax_error!(format!("{:?}", expected), "None"),
        }
    }

    fn parse_abstraction(&mut self) -> Result<Expr> {
        if let Some(TokenKind::Identifier(param)) = self.consume() {
            self.consume_expect(TokenKind::Operator(Operator::Dot));
            let body = self.parse_expression(Precedence::Lowest);
            Ok(Expr::Abstraction {
                param: param,
                body: Box::new(body?),
            })
        } else {
            throw_syntax_error!("parameter", "none")
        }
    }
    fn parse_recursion(&mut self) -> Result<Expr> {
        if let Some(TokenKind::Operator(Operator::LeftParen)) = self.consume() {
            let body = self.parse_expression(Precedence::Lowest);
            self.consume_expect(TokenKind::Operator(Operator::RightParen));
            Ok(Expr::Recursion(Box::new(body?)))
        } else {
            throw_syntax_error!("parameter", "none")
        }
    }
    fn parse_infix(&mut self, left: Expr) -> Result<Expr> {
        match self.consume() {
            Some(TokenKind::Operator(Operator::Plus)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::Add,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Sum)?),
            }),
            Some(TokenKind::Operator(Operator::Minus)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::Sub,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Sum)?),
            }),
            Some(TokenKind::Operator(Operator::Asterisk)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::Mul,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Product)?),
            }),
            Some(TokenKind::Operator(Operator::Slash)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::Div,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Product)?),
            }),
            Some(TokenKind::Operator(Operator::BitAnd)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::BitAnd,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Bitwise)?),
            }),
            Some(TokenKind::Operator(Operator::BitOr)) => Ok(Expr::BinaryOperation {
                op: BinaryOp::BitOr,
                lhs: Box::new(left),
                rhs: Box::new(self.parse_expression(Precedence::Bitwise)?),
            }),
            _ => bail!(
                "Unexpected token in infix position, Did you forget to pass parameter to application?"
            ),
        }
    }

    fn look_ahead(&self) -> Option<&TokenKind> {
        self.tokens.last()
    }

    fn consume(&mut self) -> Option<TokenKind> {
        self.tokens.pop()
    }

    fn get_precedence(&self, token: &TokenKind) -> Precedence {
        match token {
            TokenKind::Operator(Operator::Plus) | TokenKind::Operator(Operator::Minus) => {
                Precedence::Sum
            }
            TokenKind::Operator(Operator::Asterisk) | TokenKind::Operator(Operator::Slash) => {
                Precedence::Product
            }
            TokenKind::Operator(Operator::BitAnd) | TokenKind::Operator(Operator::BitOr) => {
                Precedence::Bitwise
            }
            // TokenKind::Operator(Operator::LeftParen) => Precedence::Lowest,
            // TokenKind::Operator(Operator::LeftParen) => Precedence::Lowest,
            _ => Precedence::Lowest,
        }
    }
}
