use crate::utils::{LexerError, LexerState, Token};

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
