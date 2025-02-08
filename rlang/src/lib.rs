pub mod environment;
pub mod expr;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod stmt;
pub mod resolver;

pub fn run_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = interpreter::Interpreter::new();
    let contents = std::fs::read_to_string(path)?;
    match run(&mut interpreter, &contents) {
        Err(msg) => Err(msg.into()),
        Ok(()) => Ok(()),
    }
}

pub fn run(interpreter: &mut interpreter::Interpreter, contents: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(contents);
    let tokens = lexer.scan_tokens()?;

    let mut parser = parser::Parser::new(tokens.to_vec());
    let stmts = parser.parse()?;
    interpreter.interpret(stmts.iter().map(|b| b.as_ref()).collect())?;

    Ok(())
}
