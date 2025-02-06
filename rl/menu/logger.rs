use std::{
    collections::HashMap,
    io::{self, Write},
};

#[allow(dead_code)]
#[derive(Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum LoggerType {
    Error,
    Info,
    Message,
}

#[cfg(windows)]
fn enable_ansi_support() -> bool {
    use std::ptr;
    use winapi::um::consoleapi::SetConsoleMode;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut mode = 0;
        if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) == 0 {
            return false;
        }
        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0
    }
}

pub struct Logger<'a> {
    logs: HashMap<LoggerType, Vec<&'a str>>,
}

impl<'a> Logger<'a> {
    pub fn new() -> Self {
        #[cfg(windows)]
        enable_ansi_support();

        Self {
            logs: HashMap::new(),
        }
    }

    pub fn log_msg(&mut self, msg: &'a str, kind: LoggerType) {
        self.logs.entry(kind).or_insert_with(Vec::new).push(msg);
    }

    pub fn print_logs(self) {
        let mut stdout = io::stdout();
        self.logs.into_iter().for_each(|(item, messages)| {
            for msg in messages {
                match (&item, msg) {
                    (LoggerType::Info, msg) => {
                        writeln!(stdout, "\x1b[1;36m[#] \x1b[0;36m{}\x1b[0m", msg).unwrap()
                    }
                    (LoggerType::Error, msg) => {
                        writeln!(stdout, "\x1b[1;31m[*] \x1b[0;31m{}\x1b[0m", msg).unwrap()
                    }
                    (LoggerType::Message, msg) => {
                        writeln!(stdout, "\x1b[1;32m[+] \x1b[0;32m{}\x1b[0m", msg).unwrap()
                    }
                }
            }
        });
    }
}
