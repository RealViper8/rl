use crate::{
    environment::Environment,
    lexer::{self, Token, TokenType},
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum LiteralValue {
    Number(f32),
    StringValue(String),
    True,
    False,
    Nil,
}

impl LiteralValue {
    pub fn is_falsy(&self) -> LiteralValue {
        match self {
            Self::Number(x) => {
                if *x == 0.0 {
                    Self::True
                } else {
                    Self::False
                }
            }
            Self::StringValue(s) => {
                if s.len() == 0 {
                    Self::True
                } else {
                    Self::False
                }
            }
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Nil => Self::True,
        }
    }
    pub fn is_truthy(&self) -> LiteralValue {
        match self {
            Self::Number(x) => {
                if *x == 0.0 as f32 {
                    Self::False
                } else {
                    Self::True
                }
            }
            Self::StringValue(s) => {
                if s.len() == 0 {
                    Self::False
                } else {
                    Self::True
                }
            }
            Self::True => Self::True,
            Self::False => Self::False,
            Self::Nil => Self::False,
        }
    }
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: Cow<str> = match self {
            Self::Number(x) => Cow::Owned(x.to_string()),
            Self::StringValue(x) => Cow::Borrowed(x),
            Self::True => Cow::Borrowed("true"),
            Self::False => Cow::Borrowed("false"),
            Self::Nil => Cow::Borrowed("nil"),
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Literal {
        value: LiteralValue,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}

impl Expr {
    pub fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        match self {
            Expr::Assign { name, value } => {
                let new_value = (*value).evaluate(environment.clone())?;
                let assign_success = environment
                    .borrow_mut()
                    .assign(&name.lexme, new_value.clone());
                if assign_success {
                    Ok(new_value)
                } else {
                    Err(format!("Variable {} has not been declared", name.lexme))
                }
            }
            Expr::Variable { name } => match environment.borrow().get(&name.lexme) {
                Some(val) => Ok(val.clone()),
                None => Err(format!("Variable '{}' has not been declared", name.lexme)),
            },
            Expr::Literal { value } => Ok((*value).clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => match operator.token_t {
                TokenType::Or => {
                    let lhs_val = left.evaluate(environment.clone())?;
                    let lhs_true = left.evaluate(environment.clone())?.is_truthy();
                    if lhs_true == LiteralValue::True {
                        Ok(lhs_val)
                    } else {
                        right.evaluate(environment)
                    }
                }
                TokenType::And => {
                    let lhs_val = left.evaluate(environment.clone())?;
                    let lhs_true = lhs_val.is_truthy();
                    if lhs_true == LiteralValue::False {
                        Ok(lhs_true)
                    } else {
                        right.evaluate(environment)
                    }
                }
                ty => Err(format!("Invalid token in logical expression: {}", ty)),
            },
            Expr::Grouping { expression } => expression.evaluate(environment),
            Expr::Unary { operator, right } => {
                match ((*right).evaluate(environment)?, operator.token_t) {
                    (LiteralValue::Number(x), TokenType::Minus) => Ok(LiteralValue::Number(-x)),
                    (_, TokenType::Minus) => Err(format!("Minus not implemented for {}", right)),
                    (any, TokenType::Bang) => Ok(any.is_falsy()),
                    (_, ttype) => Err(format!("{} is not valid unary operator", ttype)),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(environment.clone())?;
                let right = right.evaluate(environment)?;

                match (&left, operator.token_t, &right) {
                    (LiteralValue::Number(x), TokenType::Plus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x + y))
                    }
                    (LiteralValue::Number(x), TokenType::Minus, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x - y))
                    }
                    (LiteralValue::Number(x), TokenType::Slash, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x / y))
                    }
                    (LiteralValue::Number(x), TokenType::Less, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from(x < y))
                    }
                    (LiteralValue::Number(x), TokenType::LessEqual, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from(x <= y))
                    }
                    (LiteralValue::Number(x), TokenType::Greater, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::from(x > y))
                    }
                    (LiteralValue::Number(x), TokenType::Star, LiteralValue::Number(y)) => {
                        Ok(LiteralValue::Number(x * y))
                    }

                    (LiteralValue::StringValue(_), op, LiteralValue::Number(_)) => {
                        Err(format!("{} is not defined for String and Number", op,))
                    }
                    (LiteralValue::Number(_), op, LiteralValue::StringValue(_)) => {
                        Err(format!("{} is not defined for String and Number", op))
                    }
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Plus,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::StringValue((*s1).clone() + s2)),
                    (x, TokenType::BangEqual, y) => Ok(LiteralValue::from(x != y)),
                    (x, TokenType::EqualEqual, y) => Ok(LiteralValue::from(x == y)),

                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Greater,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from(s1 > s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::GreaterEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from(s1 >= s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::Less,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from(s1 < s2)),
                    (
                        LiteralValue::StringValue(s1),
                        TokenType::LessEqual,
                        LiteralValue::StringValue(s2),
                    ) => Ok(LiteralValue::from(s1 <= s2)),
                    (x, ttype, y) => Err(format!(
                        "{} is not implemented for the operands `{}` and `{}`",
                        ttype, x, y
                    )),
                }
            }
        }
    }
}

trait LiteralValueExt {
    fn unwrap_as_string(&self) -> Cow<str>;
    fn unwrap_as_f32(&self) -> f32;
}

impl LiteralValueExt for Option<lexer::LiteralValue> {
    fn unwrap_as_string(&self) -> Cow<str> {
        match self {
            Some(lexer::LiteralValue::StringValue(s)) => Cow::Borrowed(s),
            Some(lexer::LiteralValue::IdentifierValue(s)) => Cow::Borrowed(s),
            _ => panic!("Could not unwrap as string"),
        }
    }
    fn unwrap_as_f32(&self) -> f32 {
        match self {
            Some(lexer::LiteralValue::IntValue(s)) => *s as f32,
            Some(lexer::LiteralValue::FloatValue(s)) => *s as f32,
            _ => panic!("Could not unwrap as f32"),
        }
    }
}

impl From<Token> for LiteralValue {
    fn from(value: Token) -> Self {
        match value.token_t {
            TokenType::String => Self::StringValue(value.literal.unwrap_as_string().to_string()),
            TokenType::Number => Self::Number(value.literal.unwrap_as_f32()),

            TokenType::False => Self::False,
            TokenType::True => Self::True,
            TokenType::Nil => Self::Nil,
            _ => panic!("Could not create LiteralValue from {:?}", value),
        }
    }
}

impl From<bool> for LiteralValue {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

impl AsRef<str> for LiteralValue {
    fn as_ref(&self) -> &str {
        match self {
            &LiteralValue::StringValue(_) => "String",
            &LiteralValue::Number(_) => "Number",

            &LiteralValue::True | &LiteralValue::False => "Boolean",
            &LiteralValue::Nil => "nil",
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Logical {
                left,
                operator,
                right,
            } => format!("({} {} {})", operator, left, right),
            Self::Assign { name, value } => format!("({name} = {value})"),
            Self::Binary {
                left,
                operator,
                right,
            } => format!("({} {} {})", operator.lexme, left, right),

            Self::Grouping { expression } => format!("(group {})", expression.as_ref()),
            Self::Literal { value } => format!("{}", value),
            Self::Unary { operator, right } => {
                let operator_str = &operator.lexme;
                // let right_str = (*right).to_string();
                format!("({} {})", operator_str, right)
            }
            Expr::Variable { name } => format!("(var {name})"),
        };
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pretty_print_ast() {
        let minus_token = Token {
            token_t: TokenType::Minus,
            lexme: "-".to_string(),
            literal: None,
            line_number: 0,
        };
        let one_two_three = Expr::Literal {
            value: LiteralValue::Number(123.0),
        };
        let group = Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: LiteralValue::Number(45.67),
            }),
        };
        let multi = Token {
            token_t: crate::lexer::TokenType::Star,
            lexme: "*".to_string(),
            literal: None,
            line_number: 0,
        };
        let ast = Expr::Binary {
            left: Box::from(Expr::Unary {
                operator: minus_token,
                right: Box::from(one_two_three),
            }),
            operator: multi,
            right: Box::from(group),
        };
        let res = ast.to_string();
        assert_eq!(res, "(* (- 123) (group 45.67))");
    }
}
