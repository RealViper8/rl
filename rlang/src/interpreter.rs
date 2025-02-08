use crate::{environment::Environment, expr::LiteralValue, lexer::Token, stmt::Stmt};
use std::{cell::RefCell, rc::Rc, time::SystemTime};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

fn clock_impl(_env: Rc<RefCell<Environment>>, _args: &Vec<LiteralValue>) -> LiteralValue {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Could not get system time")
        .as_secs_f64();
    LiteralValue::Number(now)
}

impl Interpreter {
    pub fn new() -> Self {
        let mut globals = Environment::new();
        globals.define("clock".into(), LiteralValue::Callable {
            name: "clock".into(),
            arity: 0,
            fun: Rc::new(clock_impl),
        });
        Self {
            globals: Rc::new(RefCell::new(Environment::new())),
            // environment: Rc::new(RefCell::new(Environment::new())),
            environment: Rc::new(RefCell::new(globals)),
        }
    }

    fn for_closure(parent: Rc<RefCell<Environment>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        environment.borrow_mut().enclosing = Some(parent);

        Self {
            globals: Rc::new(RefCell::new(Environment::new())),
            environment,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<&Stmt>) -> Result<(), String> {
        for stmt in stmts {
            match stmt.clone() {
                Stmt::ReturnStmt { keyword: _, value } => {
                    let eval;
                    if let Some(value) = value {
                        eval = value.evaluate(self.environment.clone())?;
                    } else {
                        eval = LiteralValue::Nil;
                    }

                    self.globals
                        .borrow_mut()
                        .define_top_level("return".into(), eval);
                }
                Stmt::Function { name, params, body } => {
                    let arity = params.len();

                    let params: Vec<Token> = params.iter().map(|t| (*t).clone()).collect();
                    let body: Vec<Box<Stmt>> = body.iter().map(|b| (*b).clone()).collect();

                    let name_clone = name.lexme.clone();
                    let fun_impl = move |parent_env, args: &Vec<LiteralValue>| {
                        let mut clos_int = Interpreter::for_closure(parent_env);

                        for (i, arg) in args.iter().enumerate() {
                            clos_int
                                .environment
                                .borrow_mut()
                                .define(params[i].lexme.clone(), (*arg).clone());
                        }

                        for i in 0..(body.len()) {
                            clos_int
                                .interpret(vec![body[i].as_ref()])
                                .expect(&format!("Evaluating failed inside {}", name_clone));

                            if let Some(value) = clos_int.globals.borrow_mut().get("return") {
                                //clos_int.environment.borrow_mut().delete("return");
                                return value;
                            }

                            if let Stmt::ReturnStmt {
                                keyword: _,
                                value: _,
                            } = *body[i].clone()
                            {
                                let value = clos_int
                                    .environment
                                    .borrow()
                                    .get("return")
                                    .unwrap_or(LiteralValue::Nil);
                                return value;
                            }
                        }

                        LiteralValue::Nil
                    };

                    let callable = LiteralValue::Callable {
                        name: name.to_string(),
                        arity,
                        fun: Rc::new(fun_impl),
                    };

                    self.environment.borrow_mut().define(name.lexme, callable);
                }
                Stmt::WhileStmt { condition, body } => {
                    let mut flag = condition.evaluate(self.environment.clone())?;

                    let body = Rc::new(RefCell::new(*body));
                    while flag.is_truthy() == LiteralValue::True {
                        self.interpret(vec![&body.borrow()])?;
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
                        self.interpret(vec![&then])?;
                    } else if let Some(else_stmt) = r#else {
                        self.interpret(vec![&else_stmt])?;
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
                    let block_result =
                        self.interpret(statements.iter().map(|b| b.as_ref()).collect());
                    self.environment = old_env;

                    block_result?
                }
            };
        }

        Ok(())
    }
}
