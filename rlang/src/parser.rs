use crate::expr::{Expr, LiteralValue};
use crate::lexer::{Token, TokenType};
use crate::stmt::Stmt;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
enum FunctionKind {
    Function,
    Method,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>, String> {
        let mut stmts: Vec<Stmt> = vec![];
        let mut errs = vec![];

        while !self.is_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(s) => stmts.push(s),
                Err(msg) => {
                    errs.push(msg);
                    self.synchronize();
                }
            }
        }

        if errs.is_empty() {
            Ok(stmts.iter().map(|f| Box::new(f.clone())).collect())
        } else {
            Err(errs.join("\n"))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenType::Var) {
            self.var_declaration()
        } else if self.match_token(&TokenType::Fn) {
            self.function(FunctionKind::Function)
        } else {
            self.statement()
        }
    }

    fn function(&mut self, kind: FunctionKind) -> Result<Stmt, String> {
        let name = self.consume(TokenType::Identifier, &format!("Expected {kind:?} name"))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expected '(' after {kind:?} name"),
        )?;

        let mut params: Vec<Token> = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    let location = self.peek().line_number;
                    return Err(format!(
                        "Line {location}: Cant have more than 255 arguments"
                    ));
                }

                let param = self.consume(TokenType::Identifier, "Expected paramter name")?;
                params.push(param);

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expected '{{' {kind:?} body."),
        )?;

        let body = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!("Block statement parsed something that was not a block"),
        };

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, String> {
        let token = self.consume(TokenType::Identifier, "Expected variable name")?;
        let initializer;
        if self.match_token(&TokenType::Equal) {
            initializer = self.expression()?;
        } else {
            initializer = Expr::Literal {
                value: LiteralValue::Nil,
            };
        }

        self.consume(TokenType::Semicolon, "Expected a ';' after variable name")?;

        Ok(Stmt::Var {
            name: token,
            initializer,
        })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenType::Print) {
            self.print_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            self.block_statement()
        } else if self.match_token(&TokenType::If) {
            self.if_statement()
        } else if self.match_token(&TokenType::While) {
            self.while_statement()
        } else if self.match_token(&TokenType::For) {
            self.for_statement()
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let keyword = self.previous();
        let value;

        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        } else {
            value = None;
        }

        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;

        Ok(Stmt::ReturnStmt { keyword, value })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after for")?;

        let initializer: Option<Stmt>;
        if self.match_token(&TokenType::Semicolon) {
            initializer = None;
        } else if self.match_token(&TokenType::Var) {
            let var_decl = self.var_declaration()?;
            initializer = Some(var_decl);
        } else {
            let expr = self.expression_statement()?;
            initializer = Some(expr);
        }

        let condition: Option<Expr>;
        if !self.check(TokenType::Semicolon) {
            let expr = self.expression()?;
            condition = Some(expr);
        } else {
            condition = None;
        }

        self.consume(TokenType::Semicolon, "Expected ';' after loop condition")?;

        let increment: Option<Expr>;
        if !self.check(TokenType::RightParen) {
            let expr = self.expression()?;
            increment = Some(expr);
        } else {
            increment = None;
        }

        self.consume(TokenType::RightParen, "Expected ')' after for clauses")?;

        let mut body = self.statement()?;
        if let Some(then) = increment {
            body = Stmt::Block {
                statements: vec![
                    Box::new(body),
                    Box::new(Stmt::Expression { expression: then }),
                ],
            }
        }

        let cond;
        match condition {
            None => {
                cond = Expr::Literal {
                    value: LiteralValue::True,
                }
            }
            Some(c) => cond = c,
        }

        body = Stmt::WhileStmt {
            condition: cond,
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block {
                statements: vec![Box::new(init), Box::new(body)],
            }
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after while")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;
        let body = self.statement()?;

        Ok(Stmt::WhileStmt {
            condition,
            body: Box::new(body),
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after if-statement")?;
        let predicate = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if-predicate")?;
        let then = Box::new(self.statement()?);
        let r#else = if self.match_token(&TokenType::Else) {
            let stm = self.statement()?;
            Some(Box::new(stm))
        } else {
            None
        };

        Ok(Stmt::IfStmt {
            predicate,
            then,
            r#else,
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_end() {
            let decl = self.declaration()?;
            statements.push(decl);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after a block")?;
        Ok(Stmt::Block {
            statements: statements
                .iter()
                .map(|item| Box::new(item.clone()))
                .collect::<Vec<Box<Stmt>>>(),
        })
    }

    fn check(&mut self, ty: TokenType) -> bool {
        self.peek().token_t == ty
    }

    fn print_statement(&mut self) -> Result<Stmt, String> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression { expression: expr })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        // if self.match_token(&TokenType::Fn) {
        //     self.function_expression()
        // } else {
        //     self.assignment()
        // }
        self.assignment()
    }

    fn function_expression(&mut self) -> Result<Expr, String> {
        let paren = self.consume(
            TokenType::LeftParen,
            "Expected '(' after anonymous function",
        )?;
        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    let location = self.peek().line_number;
                    return Err(format!(
                        "line {location}: Cant have more than 255 arguments"
                    ));
                }

                let param = self.consume(TokenType::Identifier, "Expected parameter name")?;
                parameters.push(param);

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after anonymous function paramters",
        )?;
        self.consume(
            TokenType::LeftBrace,
            "Expected '{' after anonymous function decleration",
        )?;

        let body = match self.block_statement()? {
            Stmt::Block { statements } => statements,
            _ => panic!(
                "Drink iced coffee panic attack (Block statement parsed something that was not a block)"
            ),
        };

        Ok(Expr::AnonFunction {
            paren,
            arguments: parameters,
            body,
        })
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(&TokenType::Equal) {
            let value = self.assignment()?;

            match expr {
                Expr::Variable { name } => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    });
                }
                _ => return Err("Invalid assignment target.".into()),
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(&TokenType::Or) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenType::And) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let rhs = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(rhs),
            };
        }

        Ok(expr)
    }

    fn advance(&mut self) -> Token {
        if !self.is_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let rhs = self.term()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let rhs = self.factor()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        while self.match_tokens(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let rhs = self.unary()?;
            expr = Expr::Binary {
                left: Box::from(expr),
                operator: op,
                right: Box::from(rhs),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let rhs = self.unary()?;
            Ok(Expr::Unary {
                operator: op,
                right: Box::from(rhs),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = vec![];

        if !self.check(TokenType::RightParen) {
            loop {
                let arg = self.expression()?;
                arguments.push(arg);
                if arguments.len() >= 255 {
                    let location = self.peek().line_number;
                    return Err(format!(
                        "Line {location}: Cant have more than 255 arguments"
                    ));
                }

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn consume(&mut self, token_t: TokenType, msg: &str) -> Result<Token, String> {
        let token = self.peek();
        if token.token_t == token_t {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            Err(msg.into())
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek();
        let result: Expr;
        match token.token_t {
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')'")?;
                result = Expr::Grouping {
                    expression: Box::from(expr),
                }
            }
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number
            | TokenType::String => {
                self.advance();
                result = Expr::Literal {
                    value: LiteralValue::from(token),
                }
            }
            TokenType::Fn => {
                self.advance();
                result = self.function_expression()?;
            }
            TokenType::Identifier => {
                self.advance();
                result = Expr::Variable {
                    name: self.previous(),
                };
            }
            _ => return Err("Expected expression".into()),
        }

        Ok(result)
        // if self.match_token(&TokenType::LeftParen) {
        // } else if self.match_token(&TokenType::False) {
        //     let token = self.peek();
        //     self.advance();
        //     Ok(Expr::Literal {
        //         value: LiteralValue::from(token),
        //     })
        // }
    }

    fn peek(&mut self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&mut self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_end(&mut self) -> bool {
        self.peek().token_t == TokenType::Eof
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_end() {
            if self.previous().token_t == TokenType::Semicolon {
                return;
            }

            match self.peek().token_t {
                TokenType::Class
                | TokenType::Fn
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn match_token(&mut self, ty: &TokenType) -> bool {
        if self.is_end() {
            false
        } else {
            if self.peek().token_t == *ty {
                self.advance();
                true
            } else {
                false
            }
        }
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.match_token(ty) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{self, Lexer};

    use super::*;

    #[test]
    fn test_addition() {
        let one = Token {
            token_t: TokenType::Number,
            lexme: "1".to_string(),
            literal: Some(lexer::LiteralValue::FloatValue(1.0)),
            line_number: 0,
        };
        let plus = Token {
            token_t: TokenType::Plus,
            lexme: "+".to_string(),
            literal: None,
            line_number: 0,
        };
        let two = Token {
            token_t: TokenType::Number,
            lexme: "2".to_string(),
            literal: Some(lexer::LiteralValue::FloatValue(2.0)),
            line_number: 0,
        };
        let semi = Token {
            token_t: TokenType::Semicolon,
            lexme: ";".to_string(),
            literal: None,
            line_number: 0,
        };
        let eof = Token {
            token_t: TokenType::Eof,
            lexme: "".to_string(),
            literal: None,
            line_number: 0,
        };

        let tokens = vec![one, plus, two, semi, eof];
        let mut parser = Parser::new(tokens);
        let parser_expr = parser.parse().unwrap();
        let str_expr = parser_expr[0].to_string();

        assert_eq!(str_expr, "(+ 1 2)");
    }

    #[test]
    fn test_comparison() {
        let source = "1 + 2 == 5 + 7;";
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens.to_vec());
        let parsed_expr = parser.parse().unwrap();
        let str_expr = parsed_expr[0].to_string();
        assert_eq!(str_expr, "(== (+ 1 2) (+ 5 7))");
    }

    #[test]
    fn test_eq_with_paren() {
        let source = "1 == (2 + 2);";
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.scan_tokens().unwrap();
        let mut parser = Parser::new(tokens.to_vec());
        let parsed_expr = parser.parse().unwrap();
        let str_expr = parsed_expr[0].to_string();
        assert_eq!(str_expr, "(== 1 (group (+ 2 2)))");
    }
}
