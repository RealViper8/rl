use menu_lib::logger::{Logger, LoggerType};
use rlang::{interpreter::Interpreter, lexer::Lexer, parser::Parser};
use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, BufReader, Write},
    process::exit,
};

#[path = "../menu/menu.rs"]
mod menu_lib;

fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::new();
    let contents = fs::read_to_string(path)?;
    match run(&mut interpreter, &contents) {
        Err(msg) => Err(msg.into()),
        Ok(()) => Ok(()),
    }
}

fn run(interpreter: &mut Interpreter, contents: &str) -> Result<(), String> {
    let mut lexer = Lexer::new(contents);
    let tokens = lexer.scan_tokens()?;

    let mut parser = Parser::new(tokens.to_vec());
    let stmts = parser.parse()?;
    interpreter.interpret(stmts)?;

    Ok(())
}

fn run_prompt() -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut reader = BufReader::new(stdin);

    let mut logger = Logger::new();
    logger.log_msg("nigma", LoggerType::Info);

    logger.print_logs();

    loop {
        print!("\x1b[1;36m> ");
        print!("\x1b[0;32m ");
        io::stdout().flush().unwrap();
        reader.read_line(&mut buffer).unwrap();

        match buffer.to_lowercase().trim() {
            "exit" | "quit" | "q" => break,
            "help" => println!("\x1b[1;36mRL"),
            _ => (),
        }

        match run(&mut interpreter, &buffer) {
            Ok(_) => (),
            Err(msg) => println!("\x1b[0;31m{}\x1b[0m", msg),
        }
        print!("\x1b[0m ");
        buffer.clear();
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&2) {
        std::cmp::Ordering::Greater => {
            eprintln!("Usage: rl [script]");
            exit(-1);
        }
        std::cmp::Ordering::Equal => match run_file(&args[1]) {
            Err(msg) => println!("Error: {}", msg),
            Ok(_) => exit(0),
        },
        _ => match run_prompt() {
            Ok(_) => (),
            Err(msg) => println!("Error: {}", msg),
        },
    }
}
