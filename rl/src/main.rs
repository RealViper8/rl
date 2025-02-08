use menu_lib::logger::{Logger, LoggerType};
use rlang::{interpreter::Interpreter, run, run_file};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    process::exit,
};

#[path = "../menu/menu.rs"]
mod menu_lib;

const DISPLAY_FUNC: &dyn Fn() -> () = &|| {
    println!(
        "\x1b[1;36mRL:
\x1b[0;32mhelp\x1b[1;35m:\t\t\x1b[0;36mShows this
\x1b[0;32mquit \x1b[1;35m| \x1b[0;32mq \x1b[1;35m| \x1b[0;32mexit\x1b[1;35m:\t\x1b[0;36mQuits the Interpreter
\x1b[0;32mhelp\x1b[1;35m:\t\t\x1b[0;36mShows this"
    );
};

fn run_prompt() -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut buffer = String::new();
    let mut reader = BufReader::new(stdin);

    let mut logger = Logger::new();
    logger.log_msg("RL Script Interpreter [V 0.1]", LoggerType::Info);

    logger.print_logs();

    loop {
        print!("\x1b[1;36m> ");
        print!("\x1b[0;32m ");
        io::stdout().flush().unwrap();
        reader.read_line(&mut buffer).unwrap();

        match buffer.to_lowercase().trim() {
            "exit" | "quit" | "q" => break,
            "help" | "?" => {
                DISPLAY_FUNC();
                buffer.clear();
                continue;
            }
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
