use std::{collections::HashMap, env::home_dir, ffi::CString, io::Error, process};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    StringLiteral(String),
    PIPE,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
    Semicolon,
    Background,
    AndIf,
    OrIf,
    EOF,
}

#[derive(Debug)]
pub enum Command {
    Simple(SimpleCommand),
    Pipe {
        left: Box<Command>,
        right: Box<Command>,
    },
    Redirect {
        child_command: Box<Command>,
        redirect_type: RedirectionType,
        target_file: String,
    },
    Sequence {
        first: Box<Command>,
        second: Box<Command>,
    },
    Background {
        child_command: Box<Command>,
    },
    Conditional {
        left: Box<Command>,
        right: Box<Command>,
        operator: ConditionalType,
    },
}

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub command: String,
    pub arguments: Vec<String>,
}

#[derive(Debug)]
pub enum RedirectionType {
    In,
    Out,
    Append,
}

#[derive(Debug)]
pub enum ConditionalType {
    And,
    Or,
}

pub enum WordSource {
    Quoted,
    Unquoted,
}

#[derive(Debug, PartialEq)]
pub enum LexerState {
    Idle,
    ReadingLiteral,
    ReadingWord,
    ReadingOtherTokens,
}

#[derive(Debug)]
pub enum LexerError {
    UnexpectedCharacter(char),
    UnterminatedStringLiteral,
    IncompleteEscapeSequence,
    IncompleteSpecialToken,
}

pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    pub built_ins: HashMap<String, Box<dyn Fn(SimpleCommand) -> Result<i32, ExecutionError>>>,
    pub shell_pgid: u32,
    pub last_exit_status: i32,
}

#[derive(Debug)]
pub enum ExecutionError {
    ForkFailed,
    Panic,
    InvalidNumberOfArgs,
    NoHomeDirectory,
    FileError(Error),
}

#[derive(Debug)]
pub enum ParserError {}

#[derive(Debug)]
pub enum ShellError {
    Lexer(LexerError),
    Parser(ParserError),
    Executor(ExecutionError),
}

impl From<LexerError> for ShellError {
    fn from(error: LexerError) -> ShellError {
        return ShellError::Lexer(error);
    }
}

impl From<ParserError> for ShellError {
    fn from(error: ParserError) -> ShellError {
        return ShellError::Parser(error);
    }
}

impl From<ExecutionError> for ShellError {
    fn from(error: ExecutionError) -> ShellError {
        return ShellError::Executor(error);
    }
}

pub fn built_ins() -> HashMap<String, Box<dyn Fn(SimpleCommand) -> Result<i32, ExecutionError>>> {
    let mut map: HashMap<String, Box<dyn Fn(SimpleCommand) -> Result<i32, ExecutionError>>> =
        HashMap::new();

    let exit = |_| process::exit(1);
    map.insert("exit".to_string(), Box::new(exit));

    let cd = |cmd: SimpleCommand| -> Result<i32, ExecutionError> {
        if cmd.arguments.len() != 2 {
            return Err(ExecutionError::InvalidNumberOfArgs);
        }
        let mut dir = cmd.arguments[1].clone();

        if dir == "~" {
            dir = match home_dir() {
                Some(t) => t.to_string_lossy().to_string(),
                None => return Err(ExecutionError::NoHomeDirectory),
            }
        }

        let c_dir = CString::new(dir).map_err(|_| ExecutionError::Panic)?;
        let c_dir = c_dir.as_ptr() as *const i8;

        unsafe { Ok(libc::chdir(c_dir)) }
    };
    map.insert("cd".to_string(), Box::new(cd));

    return map;
}

pub fn ignore_signals() {
    unsafe {
        let signals_to_ignore = [
            libc::SIGINT,  // Ctrl-C
            libc::SIGQUIT, // Ctrl-\
            libc::SIGTSTP, // Ctrl-Z
            libc::SIGTTIN, // Background process trying to read from terminal
            libc::SIGTTOU, // Background process trying to write to terminal
        ];

        let mut action: libc::sigaction = std::mem::zeroed();

        action.sa_sigaction = libc::SIG_IGN;

        for &signal in &signals_to_ignore {
            if libc::sigaction(signal, &action, std::ptr::null_mut()) == -1 {
                panic!("Failed to set signal handler for signal {}", signal);
            }
        }
    }
}
