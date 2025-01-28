//#![allow(dead_code)]

use std::{cell::LazyCell, collections::HashMap, rc::Rc};

fn is_digit(ch: char) -> bool {
    ch as u8 >= b'0' && ch as u8 <= b'9'
    //ch.is_ascii_digit()
}

fn is_alpha(ch: char) -> bool {
    let uch = ch as u8;
    (uch >= b'a' && uch <= b'z') || (uch >= b'A' && uch <= b'Z') || (ch == '_')
}

fn is_alphanum(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}

// TODO: Improve this to make runtime faster. HashMap has runtime overhead
//       Maybe using BTreeMap
pub const KEYOWRDS: LazyCell<HashMap<&str, TokenType>> = LazyCell::new(|| {
    HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("while", TokenType::While),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fn", TokenType::Fn),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
    ])
});

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    tokens: Rc<Vec<Token>>,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single char tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two characters
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    True,
    Fn,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,

    Eof,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    IntValue(i64),
    FloatValue(f64),
    StringValue(String),
    IdentifierValue(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_t: TokenType,
    pub lexme: String,
    pub literal: Option<LiteralValue>,
    pub line_number: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexme: String,
        literal: Option<LiteralValue>,
        line_number: usize,
    ) -> Self {
        Self {
            token_t: token_type,
            lexme,
            line_number,
            literal,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:?}", self.token_t, self.lexme, self.literal)
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            tokens: Rc::new(vec![]),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Rc<Vec<Token>>, String> {
        let mut errors: Vec<String> = vec![];
        while !self.is_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => errors.push(msg),
            }
        }

        Rc::get_mut(&mut self.tokens)
            .expect("Failed to get mutable")
            .push(Token {
                token_t: TokenType::Eof,
                lexme: String::new(),
                literal: None,
                line_number: self.line,
            });

        if !errors.is_empty() {
            let mut joined: String = String::new();
            for error in errors.iter() {
                joined.push_str(error);
                joined.push('\n');
            }
            return Err(joined);
        }

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            '*' => self.add_token(TokenType::Star),
            ';' => self.add_token(TokenType::Semicolon),
            '/' => {
                if self.char_match('/') {
                    loop {
                        if self.peek() == '\n' || self.is_end() {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '!' => {
                let token: TokenType = if self.char_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.char_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.char_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.char_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token);
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string()?,
            c => {
                if is_digit(c) {
                    self.number()?;
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    return Err(format!("Unrecognized char at line {}: '{}'", self.line, c));
                }
            }
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while is_alphanum(self.peek()) {
            self.advance();
        }

        let keyword = &self.source[self.start..self.current];
        if let Some(ty) = HashMap::get(&KEYOWRDS, keyword) {
            self.add_token(*ty);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;

        c as char
    }

    fn add_token(&mut self, token_t: TokenType) {
        self.push_token(token_t, None);
    }

    fn push_token(&mut self, token_t: TokenType, literal: Option<LiteralValue>) {
        let text =
            // This wont work for every character...
            // String::from_utf8(self.source.as_bytes()[self.start..self.current].into()).unwrap();
            self.source[self.start..self.current].to_string();

        Rc::get_mut(&mut self.tokens)
            .expect("Failed to get Rc mutable for push_tokens")
            .push(Token {
                token_t,
                lexme: text,
                literal,
                line_number: self.line,
            });
    }

    fn char_match(&mut self, ch: char) -> bool {
        if self.is_end() {
            return false;
        }

        if self.source.as_bytes()[self.current] as char != ch {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_end() {
            return '\0';
        }

        self.source.as_bytes()[self.current] as char
    }

    fn number(&mut self) -> Result<(), String> {
        while is_digit(self.peek()) {
            self.advance();
        }
        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let substring = &self.source[self.start..self.current];
        match substring.parse::<f64>() {
            Ok(value) => self.push_token(TokenType::Number, Some(LiteralValue::FloatValue(value))),
            Err(_) => return Err(format!("Could not parse integer: {}", substring)),
        };
        Ok(())
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.as_bytes()[self.current + 1] as char
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            return Err("Unterminated string".into());
        }
        self.advance();

        let value = &self.source.as_bytes()[self.start + 1..self.current - 1];
        self.push_token(
            TokenType::String,
            Some(LiteralValue::StringValue(
                String::from_utf8(value.into()).unwrap(),
            )),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn handle_one_char_tokens() {
        let source = "(( )) }{";
        let mut lexer = Lexer::new(source);
        lexer.scan_tokens().unwrap();
        //dbg!(&lexer.tokens);
        assert_eq!(lexer.tokens.len(), 6 + 1); // Plus one because of eof
        assert_eq!(lexer.tokens[0].token_t, TokenType::LeftParen);
        assert_eq!(lexer.tokens[1].token_t, TokenType::LeftParen);

        assert_eq!(lexer.tokens[2].token_t, TokenType::RightParen);
        assert_eq!(lexer.tokens[3].token_t, TokenType::RightParen);

        assert_eq!(lexer.tokens[4].token_t, TokenType::RightBrace);
        assert_eq!(lexer.tokens[5].token_t, TokenType::LeftBrace);

        assert_eq!(lexer.tokens[6].token_t, TokenType::Eof);
    }

    #[test]
    pub fn handle_two_char_tokens() {
        let source = "! != == >=";
        let mut lexer = Lexer::new(source);
        lexer.scan_tokens().unwrap();
        //dbg!(&lexer.tokens);
        assert_eq!(lexer.tokens.len(), 4 + 1); // Plus one because of eof
        assert_eq!(lexer.tokens[0].token_t, TokenType::Bang);
        assert_eq!(lexer.tokens[1].token_t, TokenType::BangEqual);

        assert_eq!(lexer.tokens[2].token_t, TokenType::EqualEqual);
        assert_eq!(lexer.tokens[3].token_t, TokenType::GreaterEqual);

        assert_eq!(lexer.tokens[4].token_t, TokenType::Eof);
    }

    #[test]
    fn handle_string_lit() {
        let source = "\"ABC\"";
        let mut lexer = Lexer::new(source);
        lexer.scan_tokens().unwrap();
        //dbg!(&lexer.tokens);
        assert_eq!(lexer.tokens.len(), 1 + 1); // Plus one because of eof
        assert_eq!(lexer.tokens[0].token_t, TokenType::String);
        assert_eq!(
            lexer.tokens[0].literal.as_ref().unwrap(),
            &LiteralValue::StringValue("ABC".into())
        );
    }

    #[test]
    fn handle_string_lit_unterminated() {
        let source = "\"ABC";
        let mut lexer = Lexer::new(source);
        let res = lexer.scan_tokens();
        //dbg!(&res);
        assert!(res.is_err()); // Will fail because its unterminated
    }

    #[test]
    fn handle_string_lit_multiline() {
        let source = "\"ABC\nhi\"";
        let mut lexer = Lexer::new(source);
        let res = lexer.scan_tokens();
        // dbg!(&res);
        assert!(!res.is_err()); // Wont fail because it supports multiple lines !
        assert_eq!(lexer.tokens.len(), 1 + 1); // Plus one because of eof
        assert_eq!(lexer.tokens[0].token_t, TokenType::String);
        assert_eq!(
            lexer.tokens[0].literal.as_ref().unwrap(),
            &LiteralValue::StringValue("ABC\nhi".into())
        );
    }

    #[test]
    fn num_literals() {
        let source = "123.123\n321.0\n5";
        let mut lexer = Lexer::new(source);
        lexer.scan_tokens().unwrap();
        // dbg!(&lexer);
        assert_eq!(lexer.tokens.len(), 4);

        for i in 0..3 {
            assert_eq!(lexer.tokens[i].token_t, TokenType::Number);
        }

        assert_eq!(
            lexer.tokens[0].literal.as_ref().unwrap(),
            &LiteralValue::FloatValue(123.123)
        );
        assert_eq!(
            lexer.tokens[1].literal.as_ref().unwrap(),
            &LiteralValue::FloatValue(321.0)
        );
        assert_eq!(
            lexer.tokens[2].literal.as_ref().unwrap(),
            &LiteralValue::FloatValue(5.0)
        );
    }

    #[test]
    fn get_ident() {
        let source = "this_is_var = 12;";
        let mut lexer = Lexer::new(source);
        lexer.scan_tokens().unwrap();
        // dbg!(&lexer);

        assert_eq!(lexer.tokens.len(), 5);

        assert_eq!(lexer.tokens[0].token_t, TokenType::Identifier);
        assert_eq!(lexer.tokens[1].token_t, TokenType::Equal);
        assert_eq!(lexer.tokens[2].token_t, TokenType::Number);
        assert_eq!(lexer.tokens[3].token_t, TokenType::Semicolon);
        assert_eq!(lexer.tokens[4].token_t, TokenType::Eof);
    }

    #[test]
    fn get_keywords() {
        let source = "var this_a_var = 12;\nwhile true { print 3 };";
        let mut lexer = Lexer::new(&source);
        lexer.scan_tokens().unwrap();

        // dbg!(&lexer);
        assert_eq!(lexer.tokens.len(), 13);

        assert_eq!(lexer.tokens[0].token_t, TokenType::Var);
        assert_eq!(lexer.tokens[1].token_t, TokenType::Identifier);
        assert_eq!(lexer.tokens[2].token_t, TokenType::Equal);
        assert_eq!(lexer.tokens[3].token_t, TokenType::Number);
        assert_eq!(lexer.tokens[4].token_t, TokenType::Semicolon);
        assert_eq!(lexer.tokens[5].token_t, TokenType::While);
        assert_eq!(lexer.tokens[6].token_t, TokenType::True);
        assert_eq!(lexer.tokens[7].token_t, TokenType::LeftBrace);
        assert_eq!(lexer.tokens[8].token_t, TokenType::Print);
        assert_eq!(lexer.tokens[9].token_t, TokenType::Number);
        assert_eq!(lexer.tokens[10].token_t, TokenType::RightBrace);
        assert_eq!(lexer.tokens[11].token_t, TokenType::Semicolon);
        assert_eq!(lexer.tokens[12].token_t, TokenType::Eof);
    }
}
