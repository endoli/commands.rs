//! # Command Tokenization
//!
//! The command parser needs to be able to tokenize commands
//! into their constituent words and whitespace.
//!
//! The `tokenizer` breaks source text into a vector of tokens
//! which can be either whitespace or a word. The tokenizer
//! handles using double quotes to provide a single token
//! which may include whitespace.
//!
//! Tokens also track their source location within the source
//! text. This allows the parser using the tokenizer to provide
//! better error highlighting and other functionality.
//!
//! # Examples
//!
//! ```
//! use commands::tokenizer::tokenize;
//!
//! if let Ok(tokens) = tokenize("word") {
//!     assert_eq!(tokens.len(), 1);
//! }
//!
//! if let Ok(tokens) = tokenize("show interface") {
//!     assert_eq!(tokens.len(), 3);
//! }
//!
//! if let Ok(tokens) = tokenize("echo -n \"a b c\"") {
//!     assert_eq!(tokens.len(), 5);
//! }
//!
//! if let Ok(tokens) = tokenize("ls My\\ Documents") {
//!     assert_eq!(tokens.len(), 3);
//!     assert_eq!(tokens[2].text, "My\\ Documents");
//! }
//! ```

use std::fmt;
use std::error::Error;

/// A position within a body of text.
///
/// The `SourceOffset` tracks 2 different ways of locating the
/// position:
///
/// * The index of the character within the body of text.
/// * The column and line number of the character.
///
/// The `SourceOffset` is typically used as a pair of offsets
/// indicating the start and end of a range of text as used
/// by the `SourceLocation`.
#[derive(Debug,PartialEq)]
pub struct SourceOffset {
    /// The index of this character within the body of text.
    pub char: usize,
    /// The line number on which this character may be found.
    pub line: usize,
    /// The column on which this character may be found.
    pub column: usize,
}

impl SourceOffset {
    /// Construct a `SourceOffset`.
    pub fn new(char: usize, line: usize, column: usize) -> SourceOffset {
        SourceOffset {
            char: char,
            line: line,
            column: column,
        }
    }
}

/// A range within a body of text.
#[derive(Debug,PartialEq)]
pub struct SourceLocation {
    /// The start of the range.
    pub start: SourceOffset,
    /// The end of the range.
    pub end: SourceOffset,
}

impl SourceLocation {
    /// Construct a `SourceLocation`.
    pub fn new(start: SourceOffset, end: SourceOffset) -> SourceLocation {
        SourceLocation {
            start: start,
            end: end,
        }
    }
}

/// Errors
#[derive(Clone,Debug)]
pub enum TokenizerError {
    /// Character not allowed here
    CharacterNotAllowedHere(usize),

    /// Special not yet implemented
    SpecialNotYetImplemented(usize),

    /// Escaping backslash at end of input
    EscapingBackslashAtEndOfInput,

    /// Unclosed double quote at end of input
    UnclosedDoubleQuoteAtEndOfInput,

    /// Escaped double quote at end of input
    EscapedDoubleQuoteAtEndOfInput,
}

impl Error for TokenizerError {
    fn description(&self) -> &str {
        match *self {
            TokenizerError::CharacterNotAllowedHere(_) => "Character not allowed here",
            TokenizerError::SpecialNotYetImplemented(_) => "Special not yet implemented",
            TokenizerError::EscapingBackslashAtEndOfInput => "Escaping backlash at end of input",
            TokenizerError::UnclosedDoubleQuoteAtEndOfInput => "Unclosed double quote at end of input",
            TokenizerError::EscapedDoubleQuoteAtEndOfInput => "Escaped double quote at end of input",
        }
    }
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

/// The role that a token plays: `Whitespace` or `Word`.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum TokenType {
    /// Internal usage only.
    Invalid,
    /// The token represents whitespace and not a word.
    Whitespace,
    /// The token represents a word within the body of text. This
    /// takes double quoted strings into account.
    Word,
}

/// A token from a body of text.
///
/// The lifetime parameter `'t` refers to the lifetime
/// of the body of text that was tokenized, creating this token.
#[derive(Debug,PartialEq)]
pub struct Token<'t> {
    /// The text of the token.
    pub text: &'t str,
    /// The type of the token (`Whitespace` or `Word`).
    pub token_type: TokenType,
    /// The location of the token in the source body of text.
    pub location: SourceLocation,
}

impl<'t> Token<'t> {
    /// Construct a `Token`. The lifetime parameter `'t` refers to the lifetime of the
    /// text being tokenized.
    pub fn new(text: &'t str, token_type: TokenType, location: SourceLocation) -> Token {
        Token {
            text: text,
            token_type: token_type,
            location: location,
        }
    }
}

#[derive(Clone,Copy,PartialEq)]
enum State {
    Initial,
    Special,
    Whitespace,
    Doublequote,
    DoublequoteBackslash,
    Word,
    WordBackslash,
}

struct Tokenizer<'t> {
    text: &'t str,
    state: State,
    token_type: TokenType,
    token_start: usize,
    token_end: usize,
    tokens: Vec<Token<'t>>,
}

impl<'t> Tokenizer<'t> {
    fn new(text: &'t str) -> Tokenizer {
        Tokenizer {
            text: text,
            state: State::Initial,
            token_type: TokenType::Invalid,
            token_start: 0,
            token_end: 0,
            tokens: vec![],
        }
    }

    fn reset(&mut self) {
        self.state = State::Initial;
        self.token_type = TokenType::Invalid;
        self.token_start = 0;
        self.token_end = 0;
    }

    fn reduce(&mut self) {
        let token_text = &self.text[self.token_start..self.token_end + 1];
        let loc = SourceLocation::new(SourceOffset::new(self.token_start, 0, self.token_start),
                                      SourceOffset::new(self.token_end, 0, self.token_end));
        self.tokens.push(Token::new(token_text, self.token_type, loc));
        self.reset();
    }

    fn shift(&mut self, offset: usize, next_state: State) {
        self.recognize(offset, next_state);
        self.token_end = offset;
        self.state = next_state;
    }

    fn recognize(&mut self, offset: usize, next_state: State) {
        if self.token_type == TokenType::Invalid {
            self.token_type = if next_state == State::Whitespace {
                TokenType::Whitespace
            } else {
                TokenType::Word
            };
            self.token_start = offset;
        }
    }

    fn special(&mut self, offset: usize) {
        self.shift(offset, State::Special);
        self.reduce();
    }

    fn initial(&mut self, offset: usize, c: char) {
        if c.is_whitespace() {
            self.shift(offset, State::Whitespace);
        } else if c == ';' {
            self.special(offset);
        } else if c == '?' {
            self.special(offset);
        } else if c == '|' {
            self.special(offset);
        } else if c == '"' {
            self.shift(offset, State::Doublequote);
        } else if c == '\\' {
            self.recognize(offset, State::Word);
            self.shift(offset, State::WordBackslash);
        } else {
            self.shift(offset, State::Word);
        }
    }

    fn tokenize(&mut self) -> Result<(), TokenizerError> {
        for (offset, c) in self.text.chars().enumerate() {
            match self.state {
                State::Initial => self.initial(offset, c),
                State::Whitespace => {
                    if c.is_whitespace() {
                        self.shift(offset, State::Whitespace);
                    } else {
                        self.reduce();
                        self.initial(offset, c);
                    };
                }
                State::Word => {
                    if c.is_whitespace() {
                        self.reduce();
                        self.shift(offset, State::Whitespace);
                    } else if c == ';' {
                        self.reduce();
                        self.special(offset);
                    } else if c == '|' {
                        self.reduce();
                        self.special(offset);
                    } else if c == '"' {
                        self.reduce();
                        self.shift(offset, State::Doublequote);
                    } else if c == '\\' {
                        self.shift(offset, State::WordBackslash);
                    } else {
                        self.shift(offset, State::Word);
                    }
                }
                State::WordBackslash => {
                    // XXX: This should be if !c.is_control() perhaps?
                    if c.is_alphanumeric() || c.is_whitespace() {
                        self.shift(offset, State::Word);
                    } else {
                        return Err(TokenizerError::CharacterNotAllowedHere(offset));
                    };
                }
                State::Doublequote => {
                    if c == '"' {
                        self.shift(offset, State::Doublequote);
                        self.reduce();
                    } else if c == '\\' {
                        self.shift(offset, State::DoublequoteBackslash);
                    } else {
                        self.shift(offset, State::Doublequote);
                    };
                }
                State::DoublequoteBackslash => {
                    if !c.is_whitespace() {
                        self.shift(offset, State::Doublequote);
                    } else {
                        return Err(TokenizerError::CharacterNotAllowedHere(offset));
                    };
                }
                State::Special => {
                    return Err(TokenizerError::SpecialNotYetImplemented(offset));
                }
            }
        }

        // Now for the end of the text...
        match self.state {
            State::Initial => {}
            State::Word => self.reduce(),
            State::Whitespace => self.reduce(),
            State::WordBackslash => return Err(TokenizerError::EscapingBackslashAtEndOfInput),
            State::Doublequote => return Err(TokenizerError::UnclosedDoubleQuoteAtEndOfInput),
            State::DoublequoteBackslash => {
                return Err(TokenizerError::EscapedDoubleQuoteAtEndOfInput)
            }
            State::Special => {
                return Err(TokenizerError::SpecialNotYetImplemented(self.text.len() - 1))
            }
        }

        Ok(())
    }
}

/// Tokenize a body of text.
pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizerError> {
    let mut tokenizer = Tokenizer::new(text);
    match tokenizer.tokenize() {
        Ok(_) => Ok(tokenizer.tokens),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mk_token(text: &str, token_type: TokenType, start: usize, end: usize) -> Token {
        Token::new(text,
                   token_type,
                   SourceLocation::new(SourceOffset::new(start, 0, start),
                                       SourceOffset::new(end, 0, end)))
    }

    #[test]
    fn empty_test() {
        match tokenize("") {
            Ok(ts) => assert_eq!(ts.len(), 0),
            _ => {}
        };
    }

    #[test]
    fn single_word() {
        match tokenize("a") {
            Ok(ts) => {
                assert_eq!(ts.len(), 1);
                assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
            }
            _ => {}
        };
    }

    #[test]
    fn multiple_words() {
        match tokenize(" aa bb  ccc ") {
            Ok(ts) => {
                assert_eq!(ts.len(), 7);
                assert_eq!(ts[0], mk_token(" ", TokenType::Whitespace, 0, 0));
                assert_eq!(ts[1], mk_token("aa", TokenType::Word, 1, 2));
                assert_eq!(ts[2], mk_token(" ", TokenType::Whitespace, 3, 3));
                assert_eq!(ts[3], mk_token("bb", TokenType::Word, 4, 5));
                assert_eq!(ts[4], mk_token("  ", TokenType::Whitespace, 6, 7));
                assert_eq!(ts[5], mk_token("ccc", TokenType::Word, 8, 10));
                assert_eq!(ts[6], mk_token(" ", TokenType::Whitespace, 11, 11));
            }
            _ => {}
        };
    }

    #[test]
    fn quoted_text() {
        match tokenize("a \"b c\"") {
            Ok(ts) => {
                assert_eq!(ts.len(), 3);
                assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
                assert_eq!(ts[1], mk_token(" ", TokenType::Whitespace, 1, 1));
                assert_eq!(ts[2], mk_token("\"b c\"", TokenType::Word, 2, 6));
            }
            _ => {}
        };
    }

    #[test]
    fn escaped_whitespace_in_word() {
        match tokenize("a\\ b") {
            Ok(ts) => {
                assert_eq!(ts.len(), 1);
                assert_eq!(ts[0], mk_token("a\\ b", TokenType::Word, 0, 3));
            }
            _ => {}
        };
    }
}
