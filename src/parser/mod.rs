use crate::utils::{Command, ConditionalType, RedirectionType, SimpleCommand, Token, WordSource};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
}

impl Parser {
    pub fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    pub fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        if !self.at_end() {
            self.position += 1;
        }
        token
    }

    pub fn consume_word_or_literal(&mut self) -> Option<(String, WordSource)> {
        match self.peek().clone() {
            Token::Word(word) => {
                self.advance();
                return Some((word, WordSource::Unquoted));
            }
            Token::StringLiteral(literal) => {
                self.advance();
                return Some((literal, WordSource::Quoted));
            }
            _ => None,
        }
    }

    pub fn is_redirection(&self) -> bool {
        let peeked = self.peek();
        if *peeked == Token::RedirectAppend
            || *peeked == Token::RedirectIn
            || *peeked == Token::RedirectOut
        {
            return true;
        } else {
            return false;
        }
    }

    pub fn is_pipe(&self) -> bool {
        self.peek() == &Token::PIPE
    }

    fn at_end(&self) -> bool {
        self.position == self.tokens.len() - 1
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Command, ()> {
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    parse_sequence(&mut parser)
}

pub fn parse_simple_command(parser: &mut Parser) -> Result<Command, ()> {
    let simple_command = match parser.consume_word_or_literal() {
        Some((cmd, _)) => cmd,
        None => return Err(()),
    };

    let mut arguments = vec![simple_command.clone()];

    while let Some((arg, _)) = parser.consume_word_or_literal() {
        arguments.push(arg);
    }

    Ok(Command::Simple(SimpleCommand {
        command: simple_command,
        arguments,
    }))
}

pub fn parse_redirect(parser: &mut Parser) -> Result<Command, ()> {
    let mut child_command = parse_simple_command(parser)?;

    while parser.is_redirection() {
        let redirection = parser.advance();

        let (filename, _) = parser.consume_word_or_literal().ok_or(())?;

        let redirect_type = match redirection {
            Token::RedirectIn => RedirectionType::In,
            Token::RedirectOut => RedirectionType::Out,
            Token::RedirectAppend => RedirectionType::Append,
            _ => unreachable!(),
        };

        child_command = Command::Redirect {
            child_command: Box::new(child_command),
            redirect_type,
            target_file: filename,
        };
    }

    Ok(child_command)
}

pub fn parse_pipe(parser: &mut Parser) -> Result<Command, ()> {
    let mut left = parse_redirect(parser)?;

    while parser.is_pipe() {
        parser.advance();

        let right = parse_redirect(parser)?;

        left = Command::Pipe {
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    Ok(left)
}

pub fn parse_sequence(parser: &mut Parser) -> Result<Command, ()> {
    let mut command = parse_pipe(parser)?;

    loop {
        match parser.peek().clone() {
            Token::Semicolon => {
                parser.advance();
                let right = parse_pipe(parser)?;
                command = Command::Sequence {
                    first: Box::new(command),
                    second: Box::new(right),
                };
            }
            Token::Background => {
                parser.advance();
                command = Command::Background {
                    child_command: Box::new(command),
                };
            }
            Token::AndIf => {
                parser.advance();
                let right = parse_pipe(parser)?;
                command = Command::Conditional {
                    left: Box::new(command),
                    right: Box::new(right),
                    operator: ConditionalType::And,
                }
            }
            Token::OrIf => {
                parser.advance();
                let right = parse_pipe(parser)?;
                command = Command::Conditional {
                    left: Box::new(command),
                    right: Box::new(right),
                    operator: ConditionalType::Or,
                }
            }
            _ => break,
        }
    }

    Ok(command)
}
