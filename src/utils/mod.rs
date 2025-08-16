use std::{collections::HashMap, io::Error};

pub const BUILT_INS: [&'static str; 2] = ["exit", "cd"];

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

#[derive(Debug)]
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
    pub shell_pgid: u32,
    pub last_exit_status: i32,
}

#[derive(Debug)]
pub enum ExecutionError {
    ForkFailed,
    Panic,
    FileError(Error),
}
