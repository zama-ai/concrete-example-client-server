use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Formatter;

use concrete::{FheUint16, FheUint3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Ident(String);

impl Ident {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// By descending order of precedence
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Multiplication,
    Division,
    Addition,
    Subtraction,
    BitLeftShift,
    BitRightShift,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    BitAnd,
    BitOr,
    BitXor,
}

impl Operator {
    // https://en.cppreference.com/w/c/language/operator_precedence
    fn precedence(self) -> usize {
        match self {
            Self::Multiplication | Self::Division => 7,
            Self::Addition | Self::Subtraction => 6,
            Self::BitRightShift | Self::BitLeftShift => 5,
            Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::EqualEqual => 5,
            Operator::BitAnd => 3,
            Operator::BitXor => 2,
            Operator::BitOr => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Operator(Operator),
    Ident(Ident),
    OpeningParen,
    ClosingParen,
}

fn is_delimiter_char(char: char) -> bool {
    matches!(
        char,
        '+' | '-' | '*' | '/' | '&' | '|' | '^' | '(' | ')' | '<' | '>' | ' '
    )
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TokenizationError {
    MissingExpectedChar { expected: char, actual: char },
    IllegalChar(char),
}

impl fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenizationError::MissingExpectedChar { expected, actual } => {
                write!(f, "Expected char '{}', got '{}' instead", expected, actual)
            }
            TokenizationError::IllegalChar(c) => {
                write!(f, "Illegal char '{}'", c)
            }
        }
    }
}

pub fn tokenize(string: &str) -> Result<Vec<Token>, TokenizationError> {
    let mut tokens = vec![];

    let mut ident_buffer = String::new();
    let mut char_iter = string.chars().peekable();

    while let Some(c) = char_iter.next() {
        let maybe_nc = char_iter.peek();

        if c.is_alphabetic() {
            ident_buffer.push(c);
            let is_identifier_end = maybe_nc.copied().map(is_delimiter_char).unwrap_or(false);
            if is_identifier_end {
                tokens.push(Token::Ident(Ident(ident_buffer.clone())));
                ident_buffer.clear();
            }
            continue;
        }

        match c {
            '+' => {
                tokens.push(Token::Operator(Operator::Addition));
            }
            '-' => {
                tokens.push(Token::Operator(Operator::Subtraction));
            }
            '*' => {
                tokens.push(Token::Operator(Operator::Multiplication));
            }
            '/' => {
                tokens.push(Token::Operator(Operator::Division));
            }
            '&' => {
                tokens.push(Token::Operator(Operator::BitAnd));
            }
            '|' => {
                tokens.push(Token::Operator(Operator::BitOr));
            }
            '^' => {
                tokens.push(Token::Operator(Operator::BitXor));
            }
            '(' => {
                tokens.push(Token::OpeningParen);
            }
            ')' => {
                tokens.push(Token::ClosingParen);
            }
            '<' => match maybe_nc {
                Some('<') => {
                    tokens.push(Token::Operator(Operator::BitLeftShift));
                    char_iter.next().unwrap();
                }
                Some('=') => {
                    tokens.push(Token::Operator(Operator::LessEqual));
                    char_iter.next().unwrap();
                }
                _ => tokens.push(Token::Operator(Operator::Less)),
            },
            '>' => match maybe_nc {
                Some('>') => {
                    tokens.push(Token::Operator(Operator::BitRightShift));
                    char_iter.next().unwrap();
                }
                Some('=') => {
                    tokens.push(Token::Operator(Operator::GreaterEqual));
                    char_iter.next().unwrap();
                }
                _ => tokens.push(Token::Operator(Operator::Greater)),
            },
            '=' => match maybe_nc {
                Some('=') => {
                    tokens.push(Token::Operator(Operator::EqualEqual));
                    char_iter.next().unwrap();
                }
                Some(nc) => {
                    return Err(TokenizationError::MissingExpectedChar {
                        expected: '=',
                        actual: *nc,
                    });
                }
                None => {
                    return Err(TokenizationError::MissingExpectedChar {
                        expected: '=',
                        actual: '\0',
                    });
                }
            },
            ' ' | '\n' => {}
            _ => {
                return Err(TokenizationError::IllegalChar(c));
            }
        }
    }

    if !ident_buffer.is_empty() {
        tokens.push(Token::Ident(Ident(ident_buffer.clone())));
    }

    Ok(tokens)
}

pub fn unique_idents(mut tokens: Vec<Token>) -> Vec<Ident> {
    // BTreeSet is less useful than BtreeMap with `()` as values
    let mut unique = BTreeMap::<Ident, ()>::default();
    while let Some(token) = tokens.pop() {
        if let Token::Ident(ident) = token {
            unique.insert(ident, ());
        }
    }

    unique.into_keys().collect()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransformationError {
    UnmatchedParen,
}

impl fmt::Display for TransformationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnmatchedParen => {
                write!(f, "Unmatched set of parenthesis")
            }
        }
    }
}

impl std::error::Error for TransformationError {}

pub fn to_postfix(mut tokens: Vec<Token>) -> Result<Vec<Token>, TransformationError> {
    let mut postfix_tokens = vec![];
    let mut stack = vec![];

    tokens.reverse();

    while !tokens.is_empty() {
        let token = tokens.pop().unwrap();
        // println!("--------");
        // println!("current token: {:?}", token);
        // println!("tokens: {:?}", tokens);
        // println!("postfix: {:?}", postfix_tokens);
        // println!("stack {:?}", stack);

        match token {
            Token::Ident(_) => postfix_tokens.push(token),
            Token::OpeningParen => {
                stack.push(token);
            }
            Token::ClosingParen => loop {
                let maybe_t = stack.pop();
                let t = maybe_t.ok_or(TransformationError::UnmatchedParen)?;
                if t == Token::OpeningParen {
                    break;
                }
                postfix_tokens.push(t);
            },
            Token::Operator(op) => {
                while let Some(t) = stack.last() {
                    if let Token::Operator(op2) = t {
                        if op.precedence() > op2.precedence() {
                            break;
                        }
                    }

                    if *t == Token::OpeningParen {
                        break;
                    }

                    postfix_tokens.push(stack.pop().unwrap());
                }
                stack.push(token);
            }
        }
    }

    while let Some(t) = stack.pop() {
        if t == Token::OpeningParen || t == Token::ClosingParen {
            return Err(TransformationError::UnmatchedParen);
        }
        postfix_tokens.push(t);
    }

    Ok(postfix_tokens)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionError {
    InvalidComputation,
    OperatorNotSupported(Operator),
    VariableNotDefined(Ident),
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidComputation => {
                write!(f, "Computation is not valid")
            }
            Self::OperatorNotSupported(op) => {
                write!(f, "Operator {:?} is not supported by the type", op)
            }
            Self::VariableNotDefined(ident) => {
                write!(f, "Variable '{}' was not defined", ident.0)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

pub trait ArithmeticType: Sized {
    fn apply_operator(&self, rhs: &Self, op: Operator) -> Option<Self>;
}

impl ArithmeticType for u32 {
    fn apply_operator(&self, rhs: &Self, op: Operator) -> Option<Self> {
        let result = match op {
            Operator::Multiplication => self * rhs,
            Operator::Division => self / rhs,
            Operator::Addition => self + rhs,
            Operator::Subtraction => self - rhs,
            Operator::BitAnd => self & rhs,
            Operator::BitOr => self | rhs,
            Operator::BitXor => self ^ rhs,
            Operator::BitLeftShift => self << rhs,
            Operator::BitRightShift => self >> rhs,
            Operator::Less => Self::from(self < rhs),
            Operator::Greater => Self::from(self > rhs),
            Operator::LessEqual => Self::from(self <= rhs),
            Operator::GreaterEqual => Self::from(self >= rhs),
            Operator::EqualEqual => Self::from(self == rhs),
        };
        Some(result)
    }
}

impl ArithmeticType for FheUint3 {
    fn apply_operator(&self, rhs: &Self, op: Operator) -> Option<Self> {
        let r = match op {
            Operator::Addition => self + rhs,
            Operator::Subtraction => self - rhs,
            Operator::Multiplication => self * rhs,
            Operator::Division => self / rhs,
            Operator::BitAnd => self & rhs,
            Operator::BitOr => self | rhs,
            Operator::BitXor => self ^ rhs,
            Operator::BitLeftShift => return None,
            Operator::BitRightShift => return None,
            Operator::Less => self.lt(rhs),
            Operator::Greater => self.gt(rhs),
            Operator::LessEqual => self.le(rhs),
            Operator::GreaterEqual => self.ge(rhs),
            Operator::EqualEqual => self.eq(rhs),
        };
        Some(r)
    }
}

impl ArithmeticType for FheUint16 {
    fn apply_operator(&self, rhs: &Self, op: Operator) -> Option<Self> {
        let r = match op {
            Operator::Addition => self + rhs,
            Operator::Subtraction => self - rhs,
            Operator::Multiplication => self * rhs,
            Operator::Division => return None,
            Operator::BitAnd => self & rhs,
            Operator::BitOr => self | rhs,
            Operator::BitXor => self ^ rhs,
            Operator::BitLeftShift => return None,
            Operator::BitRightShift => return None,
            _ => return None,
        };
        Some(r)
    }
}

pub fn to_variables_map<T>(valued_idents: Vec<(Ident, T)>) -> BTreeMap<Ident, T> {
    let mut variables = BTreeMap::<Ident, T>::default();
    for (ident, value) in valued_idents {
        variables.insert(ident, value);
    }
    variables
}

pub fn execute<T>(
    mut postfix: Vec<Token>,
    valued_idents: Vec<(Ident, T)>,
) -> Result<T, ExecutionError>
where
    T: ArithmeticType,
{
    let mut variables = to_variables_map(valued_idents);
    // Stack of variable identifier
    let mut ident_bank = vec![];
    let mut tmp_var_counter = 0;

    postfix.reverse();

    while let Some(token) = postfix.pop() {
        match token {
            Token::Ident(ident) => ident_bank.push(ident),
            Token::Operator(op) => {
                let rhs_ident = ident_bank.pop().ok_or(ExecutionError::InvalidComputation)?;
                let lhs_ident = ident_bank.pop().ok_or(ExecutionError::InvalidComputation)?;

                let lhs = variables
                    .get(&lhs_ident)
                    .ok_or(ExecutionError::VariableNotDefined(lhs_ident))?;
                let rhs = variables
                    .get(&rhs_ident)
                    .ok_or(ExecutionError::VariableNotDefined(rhs_ident))?;

                let result = lhs
                    .apply_operator(rhs, op)
                    .ok_or(ExecutionError::OperatorNotSupported(op))?;

                let new_ident = Ident(format!("{}", tmp_var_counter));
                variables.insert(new_ident.clone(), result);

                ident_bank.push(new_ident);

                tmp_var_counter += 1;
            }
            t => panic!("illegal token '{:?}' in postfix expression", t),
        }
    }

    if ident_bank.len() != 1 {
        return Err(ExecutionError::InvalidComputation);
    }

    ident_bank
        .first()
        .and_then(|ident| variables.remove(ident))
        .ok_or(ExecutionError::InvalidComputation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let variables = vec![
            (Ident("a".to_string()), 1),
            (Ident("b".to_string()), 17),
            (Ident("c".to_string()), 5),
            (Ident("d".to_string()), 3),
            (Ident("e".to_string()), 10),
        ];

        let tokens = tokenize("(a + b)").unwrap();
        println!("tokens: {:?}", tokens);
        let postfix = to_postfix(tokens).unwrap();
        println!("tokens: {:?}", postfix);
        assert_eq!(tokenize("a b +").unwrap(), postfix);
        let result = execute(postfix, variables.clone()).unwrap();
        println!("result is : {}", result);

        let tokens = tokenize("(a + b) - e").unwrap();
        println!("tokens: {:?}", tokens);
        let postfix = to_postfix(tokens).unwrap();
        println!("tokens: {:?}", postfix);
        assert_eq!(tokenize("a b + e -").unwrap(), postfix);
        let result = execute(postfix, variables.clone()).unwrap();
        println!("result is : {}", result);

        let tokens = tokenize("a * b - (c + d) + e").unwrap();
        println!("tokens: {:?}", tokens);
        let postfix = to_postfix(tokens).unwrap();
        println!("postfix tokens: {:?}", postfix);
        let expected_postfix = tokenize("a b * c d+-e+").unwrap();
        assert_eq!(expected_postfix, postfix);
        let result = execute(postfix, variables).unwrap();
        println!("result is : {}", result);
    }
}
