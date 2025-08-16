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

pub fn lexer(input: String) -> Result<Vec<Token>, LexerError> {
    let input = input.trim();
    let mut lexer_state = LexerState::Idle;
    let mut current = String::new();
    let mut tokens: Vec<Token> = Vec::new();

    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' => match lexer_state {
                LexerState::Idle => {}
                LexerState::ReadingLiteral => {
                    current.push(' ');
                }
                LexerState::ReadingWord => {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                    lexer_state = LexerState::Idle;
                }
                LexerState::ReadingOtherTokens => {
                    lexer_state = LexerState::Idle;
                }
            },
            '"' => match lexer_state {
                LexerState::ReadingLiteral => {
                    tokens.push(Token::StringLiteral(current.clone()));
                    current.clear();
                    lexer_state = LexerState::Idle;
                }
                LexerState::Idle => {
                    lexer_state = LexerState::ReadingLiteral;
                }
                LexerState::ReadingOtherTokens => {
                    lexer_state = LexerState::ReadingLiteral;
                }
                _ => {
                    return Err(LexerError::UnexpectedCharacter(ch));
                }
            },
            '\\' => match lexer_state {
                LexerState::ReadingLiteral | LexerState::ReadingWord => {
                    if let Some(escaped_char) = chars.next() {
                        let char_to_push = match escaped_char {
                            'n' => '\n',
                            't' => '\t',
                            '\\' => '\\',
                            other => other,
                        };
                        current.push(char_to_push);
                    } else {
                        return Err(LexerError::IncompleteEscapeSequence);
                    }
                }
                LexerState::Idle => {
                    if let Some(escaped_char) = chars.next() {
                        if escaped_char.is_whitespace() {
                        } else {
                            let char_to_push = match escaped_char {
                                'n' => '\n',
                                't' => '\t',
                                '\\' => '\\',
                                other => other,
                            };
                            current.push(char_to_push);
                            lexer_state = LexerState::ReadingWord;
                        }
                    } else {
                        return Err(LexerError::IncompleteEscapeSequence);
                    }
                }
                _ => {
                    return Err(LexerError::UnexpectedCharacter(ch));
                }
            },
            '>' | '&' | '|' => match lexer_state {
                LexerState::Idle => {
                    if let Some(&char) = chars.peek()
                        && char == ch
                    {
                        chars.next();
                        let token = match ch {
                            '>' => Token::RedirectAppend,
                            '&' => Token::AndIf,
                            '|' => Token::OrIf,
                            _ => unreachable!(),
                        };
                        tokens.push(token)
                    } else {
                        let token = match ch {
                            '>' => Token::RedirectOut,
                            '&' => Token::Background,
                            '|' => Token::PIPE,
                            _ => unreachable!(),
                        };
                        tokens.push(token)
                    }

                    lexer_state = LexerState::ReadingOtherTokens;
                }
                LexerState::ReadingWord => {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();

                    if let Some(&char) = chars.peek()
                        && char == ch
                    {
                        chars.next();
                        let token = match ch {
                            '>' => Token::RedirectAppend,
                            '&' => Token::AndIf,
                            '|' => Token::OrIf,
                            _ => unreachable!(),
                        };
                        tokens.push(token)
                    } else {
                        let token = match ch {
                            '>' => Token::RedirectOut,
                            '&' => Token::Background,
                            '|' => Token::PIPE,
                            _ => unreachable!(),
                        };
                        tokens.push(token)
                    }

                    lexer_state = LexerState::ReadingOtherTokens;
                }
                _ => return Err(LexerError::UnexpectedCharacter(ch)),
            },
            '<' | ';' => match lexer_state {
                LexerState::Idle => {
                    let token = match ch {
                        '<' => Token::RedirectIn,
                        ';' => Token::Semicolon,
                        _ => unreachable!(),
                    };

                    tokens.push(token);

                    lexer_state = LexerState::ReadingOtherTokens;
                }
                LexerState::ReadingWord => {
                    tokens.push(Token::Word(current.clone()));
                    current.clear();
                    let token = match ch {
                        '<' => Token::RedirectIn,
                        ';' => Token::Semicolon,
                        _ => unreachable!(),
                    };

                    tokens.push(token);

                    lexer_state = LexerState::ReadingOtherTokens;
                }
                _ => return Err(LexerError::UnexpectedCharacter(ch)),
            },
            _ => match lexer_state {
                LexerState::Idle => {
                    current.push(ch);
                    lexer_state = LexerState::ReadingWord;
                }
                LexerState::ReadingLiteral => current.push(ch),
                LexerState::ReadingWord => current.push(ch),
                _ => return Err(LexerError::UnexpectedCharacter(ch)),
            },
        }
    }

    match lexer_state {
        LexerState::ReadingWord => {
            tokens.push(Token::Word(current.clone()));
            current.clear();
        }
        LexerState::ReadingLiteral => {
            return Err(LexerError::UnterminatedStringLiteral);
        }
        LexerState::ReadingOtherTokens => {
            return Err(LexerError::IncompleteSpecialToken);
        }
        _ => {}
    }

    tokens.push(Token::EOF);

    return Ok(tokens);
}

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
