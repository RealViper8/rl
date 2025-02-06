use crate::{environment::Environment, expr::LiteralValue, stmt::Stmt};
use std::{cell::RefCell, rc::Rc};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Box<Stmt>>) -> Result<(), String> {
        for stmt in stmts {
            match *stmt {
                Stmt::ForStmt {
                    var_decl: _,
                    expr_stmt: _,
                    condition: _,
                    then: _,
                    body: _,
                } => todo!(),
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;

                    let body = Rc::new(RefCell::new(*body));
                    while flag.is_truthy() == LiteralValue::True {
                        self.interpret(vec![Box::new(body.borrow().clone())])?;
                        flag = condition.evaluate(self.environment.clone())?;
                    }
                }
                Stmt::IfStmt {
                    predicate,
                    then,
                    r#else,
                } => {
                    let truth_val = predicate.evaluate(self.environment.clone())?;
                    if truth_val.is_truthy() == LiteralValue::True {
                        self.interpret(vec![Box::new(*then)])?;
                    } else if let Some(else_stmt) = r#else {
                        self.interpret(vec![Box::new(*else_stmt)])?;
                    }
                }
                Stmt::Expression { expression } => {
                    expression.evaluate(self.environment.clone())?;
                }
                Stmt::Print { expression } => {
                    let value = expression.evaluate(self.environment.clone())?;
                    println!("{value}");
                }
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(self.environment.clone())?;

                    self.environment.borrow_mut().define(name.lexme, value);
                }
                Stmt::Block { statements } => {
                    let mut new_env = Environment::new();
                    new_env.enclosing = Some(self.environment.clone());

                    let old_env = self.environment.clone();
                    self.environment = Rc::new(new_env.into());
                    let block_result = self.interpret(statements);
                    self.environment = old_env;

                    block_result?
                }
            };
        }

        Ok(())
    }
}
