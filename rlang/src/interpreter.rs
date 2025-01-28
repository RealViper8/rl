use crate::{environment::Environment, expr::LiteralValue, stmt::Stmt};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct Interpreter {
    environment: Rc<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(Environment::new()),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Failed to get mutable ref to env"),
                    )?;

                    let body = Rc::new(RefCell::new(*body));
                    while flag.is_truthy() == LiteralValue::True {
                        self.interpret(vec![body.borrow().clone()])?;
                        flag = condition.evaluate(
                            Rc::get_mut(&mut self.environment)
                                .expect("Failed to get mutable ref to env"),
                        )?;
                    }
                }
                Stmt::IfStmt {
                    predicate,
                    then,
                    r#else,
                } => {
                    let truth_val = predicate.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Failed to get mutable env refrence"),
                    )?;
                    if truth_val.is_truthy() == LiteralValue::True {
                        self.interpret(vec![*then])?;
                    } else if let Some(else_stmt) = r#else {
                        self.interpret(vec![*else_stmt])?;
                    }
                }
                Stmt::Expression { expression } => {
                    expression.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get mutable refrence to environment"),
                    )?;
                }
                Stmt::Print { expression } => {
                    let value = expression.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get mutable refrence to environment"),
                    )?;
                    println!("{value}");
                }
                Stmt::Var { name, initializer } => {
                    let value = initializer.evaluate(
                        Rc::get_mut(&mut self.environment)
                            .expect("Could not get mutable refrence to env"),
                    )?;

                    Rc::get_mut(&mut self.environment)
                        .expect("Could not get mutable refrence to env")
                        .define(name.lexme, value);
                }
                Stmt::Block { statements } => {
                    let mut new_env = Environment::new();
                    new_env.enclosing = Some(self.environment.clone());

                    let old_env = self.environment.clone();
                    self.environment = Rc::new(new_env);
                    let block_result = self.interpret(statements);
                    self.environment = old_env;

                    block_result?
                }
            };
        }

        Ok(())
    }
}
