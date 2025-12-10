use std::{fs, io::ErrorKind, path::PathBuf};

use super::throw_lexer_syntax_error;



// Variable convection for lexer:
// prefix:"consume" => Sets (current end character of either string's or character's offset)+1;
// prefix:"look" => Doesn't increase offset;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Equal,
    LeftParen,
    RightParen,
    Dot,
    BitAnd,
    BitOr,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    Comment(String),
    Lamda,
    Recursion,
    Identifier(String),
    Literal(f64),
    Operator(Operator),
    Eof,
}

pub struct Lexer {
    tokens: Vec<char>,
    offset: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source_path: PathBuf) -> Self {
        match fs::read_to_string(source_path) {
            Ok(source) => Self {
                tokens: source.chars().into_iter().collect::<Vec<_>>(),
                offset: 0,
                row: 1,
                col: 1,
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                panic!("File not found!")
            }
            Err(e) => {
                panic!("IO Error : {:?}", e)
            }
        }
    }
    pub fn get_tokens(mut self) -> Vec<TokenKind> {
        let aprox_capacity = self
            .tokens
            .iter()
            .filter(|c| !c.is_ascii_whitespace())
            .count();
        let mut tokens = Vec::with_capacity(aprox_capacity);
        while let Some(token) = self.get_token() {
            match token {
                TokenKind::Eof => {
                    tokens.push(TokenKind::Eof);
                    break;
                }
                token => tokens.push(token),
            }
        }
        tokens
    }
    fn get_token(&mut self) -> Option<TokenKind> {
        self.consume_while(|c| c.is_whitespace());
        if let Some(ch) = self.consume() {
            match ch {
                '(' => Some(TokenKind::Operator(Operator::LeftParen)),
                ')' => Some(TokenKind::Operator(Operator::RightParen)),

                '*' => Some(TokenKind::Operator(Operator::Asterisk)),

                '/' => {
                    match self.look_ahead() {
                        Some('/') => {
                            self.advance();
                            let mut comment = self.consume_while(|c| c != '\n');
                            let last_comment_ch = comment.chars().last();
                            match last_comment_ch {
                                // some comment\r\n Something
                                Some('\r') => {
                                    comment.pop(); //remove cr
                                    Some(TokenKind::Comment(comment))
                                }
                                // some comment\n SOMETHING
                                Some(_) => Some(TokenKind::Comment(comment)),
                                // some comment\n(EOF)
                                None => Some(TokenKind::Comment(comment)),
                            }
                        }
                        // Next char should be either white space or num.
                        // Err will thrown, while building AST.
                        Some(_) => Some(TokenKind::Operator(Operator::Slash)),
                        None => Some(TokenKind::Operator(Operator::Slash)),
                    }
                }

                '-' => Some(TokenKind::Operator(Operator::Minus)),
                '+' => Some(TokenKind::Operator(Operator::Plus)),

                '=' => Some(TokenKind::Operator(Operator::Equal)),
                '.' => Some(TokenKind::Operator(Operator::Dot)),

                '&' => Some(TokenKind::Operator(Operator::BitAnd)),
                '|' => Some(TokenKind::Operator(Operator::BitOr)),

                'λ' => Some(TokenKind::Lamda),

                '𝑓' => Some(TokenKind::Recursion),

                ch => {
                    match ch {
                        ch if ch.is_ascii_alphabetic() || ch == '_' => {
                            // This is varaible
                            let mut identifier = String::from(ch);
                            identifier.push_str(
                                &self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_'),
                            );
                            Some(TokenKind::Identifier(identifier))
                        }

                        ch if ch.is_ascii_digit() => {
                            let mut numeric_literal = String::from(ch);

                            let mut last_was_underscore =
                                numeric_literal.chars().next() == Some('_');

                            let mut digit_underscore_filter = |ch: char| {
                                let is_valid = ch.is_ascii_digit() || ch == '_';
                                if last_was_underscore && ch == '_' {
                                    return false;
                                } else {
                                    last_was_underscore = ch == '_';
                                }
                                is_valid
                            };

                            numeric_literal
                                .push_str(&self.consume_while(&mut digit_underscore_filter));

                            if self.look_ahead() == Some('.') {
                                numeric_literal.push('.');
                                self.advance();
                                numeric_literal
                                    .push_str(&self.consume_while(&mut digit_underscore_filter));
                            }

                            if matches!(self.look_ahead(), Some('e') | Some('E')) {
                                numeric_literal.push('E');
                                self.advance();
                                if let Some(sign) = self.look_ahead() {
                                    if matches!(sign, '+' | '-') {
                                        numeric_literal.push(self.consume().unwrap());
                                    }
                                }
                                let digits = self.consume_while(&mut digit_underscore_filter);

                                if digits.is_empty() {
                                    throw_lexer_syntax_error!(
                                        "Standard Number",
                                        "Malformed Number",
                                        self.row,
                                        self.col
                                    );
                                }
                                numeric_literal.push_str(&digits);
                            }
                            match numeric_literal.parse::<f64>() {
                                Ok(n) => Some(TokenKind::Literal(n)),
                                Err(_) => {
                                    throw_lexer_syntax_error!(
                                        "Standard Number",
                                        "Malformed Number",
                                        self.row,
                                        self.col
                                    );
                                }
                            }
                        }
                        invalid_token => throw_lexer_syntax_error!(
                            "Valid Token",
                            invalid_token,
                            self.row,
                            self.col
                        ),
                    }
                }
            }
        } else {
            Some(TokenKind::Eof)
        }
    }

    fn consume_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut literal = String::new();
        while let Some(ch) = self.consume() {
            if !predicate(ch) {
                // For example, given ['x','y','z',' ','x1','y2','z3'], using `consume_while` with `!is_whitespace()`:
                // The end result will include up to 'z', but the offset will be set to 'x1'.
                // According to the aforementioned prefix rule, `consume` must advance by +1 from the end result,
                // i.e., the offset should point to the space " ".

                // Normalizing the offset: when the literal actually ends, the offset would have been end character's offset +2.

                self.offset = self.offset.saturating_sub(1);
                break;
            }
            literal.push(ch);
        }
        literal
    }

    fn advance(&mut self) {
        self.offset += 1
    }
    
    #[allow(unused)]
    fn advance_by(&mut self, n: usize) {
        self.offset += n;
    }

    fn get_ch_at(&mut self, n: usize) -> Option<char> {
        if n < self.tokens.len() {
            Some(self.tokens[self.offset])
        } else {
            None
        }
    }

    fn look_ahead(&mut self) -> Option<char> {
        // We are a head by 1 so, current offset is next character
        self.get_ch_at(self.offset)
    }

    #[allow(unused)]
    fn look_back(&mut self) -> Option<char> {
        // We are a head by 1 so, current offset - 1 is currently viewing character returned from consume();
        // therefore, -2
        self.get_ch_at(self.offset.saturating_sub(2))
    }

    #[allow(unused)]
    fn look_back_at(&mut self, n: usize) -> Option<char> {
        // We are a head by 1 so, current offset - 1 is currently viewing character returned from consume();
        self.get_ch_at(n.saturating_sub(3))
    }

    #[allow(unused)]
    fn look_ahead_at(&mut self, n: usize) -> Option<char> {
        // We are a head by 1 so, normalizing to intuitive/expected offset
        self.get_ch_at(n.saturating_sub(1))
    }

    fn consume(&mut self) -> Option<char> {
        // Get the current character and move next;
        let ch = self.get_ch_at(self.offset);
        self.advance();
        ch
    }
}
