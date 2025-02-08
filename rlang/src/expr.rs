use crate::{
    environment::Environment,
    interpreter::Interpreter,
    lexer::{self, Token, TokenType},
    stmt::Stmt,
};
use std::{borrow::Cow, cell::RefCell, hash::Hash, rc::Rc};

#[derive(Clone)]
pub enum LiteralValue {
    Number(f64),
    StringValue(String),
    True,
    False,
    Nil,
    Callable {
        name: String,
        arity: usize,
        fun: Rc<dyn Fn(&Vec<LiteralValue>) -> LiteralValue>,
    },
}

impl PartialEq for LiteralValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiteralValue::Number(x), LiteralValue::Number(y)) => x == y,
            (
                LiteralValue::Callable {
                    name,
                    arity,
                    fun: _,
                },
                Self::Callable {
                    name: name2,
                    arity: arity2,
                    fun: _,
                },
            ) => name == name2 && arity == arity2,
            (LiteralValue::StringValue(x), LiteralValue::StringValue(y)) => x == y,
            (LiteralValue::True, LiteralValue::True) => true,
            (LiteralValue::False, LiteralValue::False) => true,
            (LiteralValue::Nil, LiteralValue::Nil) => true,
            _ => false,
        }
    }
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
            Self::Callable {
                name: _,
                arity: _,
                fun: _,
            } => panic!("Cannot use callable as a falsy value"),
        }
    }
    pub fn is_truthy(&self) -> LiteralValue {
        match self {
            Self::Callable {
                name: _,
                arity: _,
                fun: _,
            } => panic!("Cannot use callable as a truthy value"),
            Self::Number(x) => {
                if *x == 0.0 as f64 {
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
            Self::Callable {
                name,
                arity,
                fun: _,
            } => Cow::Owned(format!("{name}{arity}")),
            Self::Number(x) => Cow::Owned(x.to_string()),
            Self::StringValue(x) => Cow::Borrowed(x),
            Self::True => Cow::Borrowed("true"),
            Self::False => Cow::Borrowed("false"),
            Self::Nil => Cow::Borrowed("nil"),
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub enum Expr {
    AnonFunction {
        paren: Token,
        arguments: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
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

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self, state)
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        let ptr = std::ptr::addr_of!(self);
        let ptr2 = std::ptr::addr_of!(other);
        ptr == ptr2
    }
}

impl Eq for Expr {}

impl Expr {
    pub fn evaluate(&self, environment: Rc<RefCell<Environment>>) -> Result<LiteralValue, String> {
        match self {
            Expr::AnonFunction {
                paren,
                arguments,
                body,
            } => {
                let arity = arguments.len();
                let env = environment.clone();
                let arguments: Vec<Token> = arguments.iter().map(|t| (*t).clone()).collect();
                let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();
                let paren = paren.clone();

                let fun_impl: Rc<dyn Fn(&Vec<LiteralValue>) -> LiteralValue> =
                    Rc::new(move |args: &Vec<LiteralValue>| {
                        let mut anon_int = Interpreter::for_anon(env.clone());
                        for (i, arg) in args.iter().enumerate() {
                            anon_int
                                .environment
                                .borrow_mut()
                                .define(arguments[i].lexme.clone(), (*arg).clone());
                        }

                        for i in 0..(body.len()) {
                            anon_int.interpret(vec![&body[i]]).expect(&format!(
                                "Evaluating failed inside anon function at line {}",
                                paren.line_number,
                            ));

                            if let Some(value) = anon_int.specials.borrow().get("return") {
                                return value;
                            }
                        }

                        LiteralValue::Nil
                    });

                Ok(LiteralValue::Callable {
                    name: "anon_function".to_string(),
                    arity,
                    fun: fun_impl,
                })
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                let callable = (*callee).evaluate(environment.clone())?;
                match callable {
                    LiteralValue::Callable { name, arity, fun } => {
                        if arguments.len() != arity {
                            return Err(format!(
                                "Callable {name} expected {arity} arguments got {}",
                                arguments.len()
                            ));
                        }
                        let mut args = vec![];
                        for arg in arguments {
                            let val = arg.evaluate(environment.clone())?;
                            args.push(val)
                        }
                        return Ok(fun(&args));
                    }
                    other => Err(format!("{} is not callable", other.as_ref()))?,
                }
                todo!()
            }
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
                    (LiteralValue::StringValue(s), TokenType::Plus, LiteralValue::Number(x)) => {
                        Ok(LiteralValue::StringValue(format!("{}{}", &s, &x)))
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

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

trait LiteralValueExt {
    fn unwrap_as_string(&self) -> Cow<str>;
    fn unwrap_as_f64(&self) -> f64;
}

impl LiteralValueExt for Option<lexer::LiteralValue> {
    fn unwrap_as_string(&self) -> Cow<str> {
        match self {
            Some(lexer::LiteralValue::StringValue(s)) => Cow::Borrowed(s),
            Some(lexer::LiteralValue::IdentifierValue(s)) => Cow::Borrowed(s),
            _ => panic!("Could not unwrap as string"),
        }
    }
    fn unwrap_as_f64(&self) -> f64 {
        match self {
            Some(lexer::LiteralValue::IntValue(s)) => *s as f64,
            Some(lexer::LiteralValue::FloatValue(s)) => *s as f64,
            _ => panic!("Could not unwrap as f32"),
        }
    }
}

impl From<Token> for LiteralValue {
    fn from(value: Token) -> Self {
        match value.token_t {
            TokenType::String => Self::StringValue(value.literal.unwrap_as_string().to_string()),
            TokenType::Number => Self::Number(value.literal.unwrap_as_f64()),

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
            &LiteralValue::Callable {
                name: _,
                arity: _,
                fun: _,
            } => "Callable",
            &LiteralValue::True | &LiteralValue::False => "Boolean",
            &LiteralValue::Nil => "nil",
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::AnonFunction {
                paren: _,
                arguments,
                body: _,
            } => format!("anon/{}", arguments.len()),
            Self::Call {
                callee,
                paren: _,
                arguments,
            } => {
                format!("({} {:?})", (*callee), arguments)
            }
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
    use std::collections::HashMap;

    #[test]
    fn expr_is_hashable() {
        let mut map = HashMap::new();
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
        let expr = Expr::Binary {
            left: Box::from(Expr::Unary {
                operator: minus_token,
                right: Box::from(one_two_three),
            }),
            operator: multi,
            right: Box::from(group),
        };
        let expr = std::rc::Rc::new(expr);
        map.insert(expr.clone(), 2);
        match map.get(&expr) {
            Some(_) => (),
            None => panic!("Unable to retrieve value"),
        }

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
        let expr = Expr::Binary {
            left: Box::from(Expr::Unary {
                operator: minus_token,
                right: Box::from(one_two_three),
            }),
            operator: multi,
            right: Box::from(group),
        };

        match map.get(&expr) {
            None => (),
            Some(_) => panic!("Test"),
        }
    }

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
