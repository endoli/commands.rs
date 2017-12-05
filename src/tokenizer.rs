// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Command Tokenization
//!
//! The [command parser] needs to be able to [tokenize] commands
//! into their constituent words and whitespace.
//!
//! The `tokenizer` breaks source text into a vector of [tokens]
//! which can be either [whitespace or a word]. The tokenizer
//! handles using single and double quotes to provide a single
//! token which may include whitespace.
//!
//! Tokens also track their [source location] within the source
//! text. This allows the parser using the tokenizer to provide
//! better error highlighting and other functionality.
//!
//! # Examples
//!
//! ```
//! use commands::tokenizer::{tokenize, TokenType};
//!
//! if let Ok(tokens) = tokenize("word") {
//!     assert_eq!(tokens.len(), 1);
//! }
//!
//! // This is 3 tokens due to the whitespace token
//! // between the 2 words.
//! if let Ok(tokens) = tokenize("show interface") {
//!     assert_eq!(tokens.len(), 3);
//!     assert_eq!(tokens[1].token_type, TokenType::Whitespace);
//! }
//!
//! // Double quoted strings are treated as a single token.
//! if let Ok(tokens) = tokenize(r#"echo -n "a b c""#) {
//!     assert_eq!(tokens.len(), 5);
//!     assert_eq!(tokens[0].text, "echo");
//!     assert_eq!(tokens[2].text, "-n");
//!     assert_eq!(tokens[4].text, r#""a b c""#);
//! }
//!
//! // Single quoted strings are treated as a single token
//! // as well.
//! if let Ok(tokens) = tokenize(r#"'"One token"' 'and another'"#) {
//!     assert_eq!(tokens.len(), 3);
//! }
//!
//! // Or you can use a \ to escape a space.
//! if let Ok(tokens) = tokenize(r#"ls My\ Documents"#) {
//!     assert_eq!(tokens.len(), 3);
//!     assert_eq!(tokens[2].text, r#"My\ Documents"#);
//! }
//! ```
//!
//! [command parser]: ../parser/index.html
//! [source location]: struct.SourceLocation.html
//! [tokenize]: fn.tokenize.html
//! [tokens]: struct.Token.html
//! [whitespace or a word]: enum.TokenType.html

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
/// by the [`SourceLocation`].
///
/// [`SourceLocation`]: struct.SourceLocation.html
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug)]
pub enum TokenizerError {
    /// Character not allowed here
    CharacterNotAllowedHere(usize),

    /// Special not yet implemented
    SpecialNotYetImplemented(usize),

    /// Escaping backslash at end of input
    EscapingBackslashAtEndOfInput,

    /// Unclosed double quote at end of input
    UnclosedDoubleQuote,

    /// Unclosed single quote at end of input
    UnclosedSingleQuote,
}

impl Error for TokenizerError {
    fn description(&self) -> &str {
        match *self {
            TokenizerError::CharacterNotAllowedHere(_) => "Character not allowed here",
            TokenizerError::SpecialNotYetImplemented(_) => "Special not yet implemented",
            TokenizerError::EscapingBackslashAtEndOfInput => "Escaping backlash at end of input",
            TokenizerError::UnclosedDoubleQuote => "Unclosed double quote at end of input",
            TokenizerError::UnclosedSingleQuote => "Unclosed single quote at end of input",
        }
    }
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

/// The role that a token plays: `Whitespace` or `Word`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    /// The token represents whitespace and not a word.
    Whitespace,
    /// The token represents a word within the body of text. This
    /// takes double quoted strings into account.
    Word,
}

/// A token from a body of text.
///
/// The lifetime parameter `'text` refers to the lifetime
/// of the body of text that was tokenized, creating this token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token<'text> {
    /// The text of the token.
    pub text: &'text str,
    /// The type of the token (`Whitespace` or `Word`).
    pub token_type: TokenType,
    /// The location of the token in the source body of text.
    pub location: SourceLocation,
}

impl<'text> Token<'text> {
    /// Construct a `Token`. The lifetime parameter `'text` refers
    /// to the lifetime of the text being tokenized.
    pub fn new(text: &'text str, token_type: TokenType, location: SourceLocation) -> Token {
        Token {
            text: text,
            token_type: token_type,
            location: location,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Initial,
    Special,
    Whitespace,
    Doublequote,
    DoublequoteBackslash,
    Singlequote,
    SinglequoteBackslash,
    Word,
    WordBackslash,
}

struct Tokenizer<'text> {
    text: &'text str,
    state: State,
    token_type: Option<TokenType>,
    token_start: usize,
    token_end: usize,
    tokens: Vec<Token<'text>>,
}

impl<'text> Tokenizer<'text> {
    fn new(text: &'text str) -> Tokenizer {
        Tokenizer {
            text: text,
            state: State::Initial,
            token_type: None,
            token_start: 0,
            token_end: 0,
            tokens: vec![],
        }
    }

    fn reset(&mut self) {
        self.state = State::Initial;
        self.token_type = None;
        self.token_start = 0;
        self.token_end = 0;
    }

    fn reduce(&mut self) {
        let token_text = &self.text[self.token_start..self.token_end + 1];
        let loc = SourceLocation::new(
            SourceOffset::new(self.token_start, 0, self.token_start),
            SourceOffset::new(self.token_end, 0, self.token_end),
        );
        self.tokens.push(Token::new(
            token_text,
            self.token_type.expect("Invalid tokenization"),
            loc,
        ));
        self.reset();
    }

    fn shift(&mut self, offset: usize, next_state: State) {
        self.recognize(offset, next_state);
        self.token_end = offset;
        self.state = next_state;
    }

    fn recognize(&mut self, offset: usize, next_state: State) {
        if self.token_type.is_none() {
            self.token_type = if next_state == State::Whitespace {
                Some(TokenType::Whitespace)
            } else {
                Some(TokenType::Word)
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
        } else if c == ';' || c == '?' || c == '|' {
            self.special(offset);
        } else if c == '"' {
            self.shift(offset, State::Doublequote);
        } else if c == '\'' {
            self.shift(offset, State::Singlequote);
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
                    } else if c == ';' || c == '|' {
                        self.reduce();
                        self.special(offset);
                    } else if c == '"' {
                        self.reduce();
                        self.shift(offset, State::Doublequote);
                    } else if c == '\'' {
                        self.reduce();
                        self.shift(offset, State::Singlequote);
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
                State::Singlequote => {
                    if c == '\'' {
                        self.shift(offset, State::Singlequote);
                        self.reduce();
                    } else if c == '\\' {
                        self.shift(offset, State::SinglequoteBackslash);
                    } else {
                        self.shift(offset, State::Singlequote);
                    };
                }
                State::SinglequoteBackslash => {
                    if !c.is_whitespace() {
                        self.shift(offset, State::Singlequote);
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
            State::Word | State::Whitespace => self.reduce(),
            State::WordBackslash => return Err(TokenizerError::EscapingBackslashAtEndOfInput),
            State::Doublequote => return Err(TokenizerError::UnclosedDoubleQuote),
            State::Singlequote => return Err(TokenizerError::UnclosedSingleQuote),
            State::DoublequoteBackslash |
            State::SinglequoteBackslash => {
                return Err(TokenizerError::EscapingBackslashAtEndOfInput)
            }
            State::Special => {
                return Err(TokenizerError::SpecialNotYetImplemented(
                    self.text.len() - 1,
                ))
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
        Token::new(
            text,
            token_type,
            SourceLocation::new(
                SourceOffset::new(start, 0, start),
                SourceOffset::new(end, 0, end),
            ),
        )
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
    fn double_quoted_text() {
        match tokenize(r#"a "b c""#) {
            Ok(ts) => {
                assert_eq!(ts.len(), 3);
                assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
                assert_eq!(ts[1], mk_token(" ", TokenType::Whitespace, 1, 1));
                assert_eq!(ts[2], mk_token(r#""b c""#, TokenType::Word, 2, 6));
            }
            _ => {}
        };
    }

    #[test]
    fn single_quoted_text() {
        match tokenize(r#"a '"b c"'"#) {
            Ok(ts) => {
                assert_eq!(ts.len(), 3);
                assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
                assert_eq!(ts[1], mk_token(" ", TokenType::Whitespace, 1, 1));
                assert_eq!(ts[2], mk_token(r#"'"b c"'"#, TokenType::Word, 2, 8));
            }
            _ => {}
        };
    }

    #[test]
    fn escaped_whitespace_in_word() {
        match tokenize(r#"a\ b"#) {
            Ok(ts) => {
                assert_eq!(ts.len(), 1);
                assert_eq!(ts[0], mk_token(r#"a\ b"#, TokenType::Word, 0, 3));
            }
            _ => {}
        };
    }

    #[test]
    fn character_not_allowed_here() {
        match tokenize(r#"ab \!"#) {
            Err(TokenizerError::CharacterNotAllowedHere(_)) => {}
            _ => panic!(),
        };

        match tokenize(r#"ab "\ ab"#) {
            Err(TokenizerError::CharacterNotAllowedHere(_)) => {}
            _ => panic!(),
        };
    }

    // TODO: Test TokenizeError::SpecialNotYetImplemented

    #[test]
    #[should_panic]
    fn escaping_backslash_at_end_of_input() {
        match tokenize(r#"ab \"#) {
            Err(TokenizerError::EscapingBackslashAtEndOfInput) => panic!(),
            _ => {}
        }
    }

    #[test]
    #[should_panic]
    fn unclosed_double_quote_at_end_of_input() {
        match tokenize(r#"ab ""#) {
            Err(TokenizerError::UnclosedDoubleQuote) => panic!(),
            _ => {}
        }
    }

    #[test]
    #[should_panic]
    fn escaped_double_quote_at_end_of_input() {
        match tokenize(r#"ab "\"#) {
            Err(TokenizerError::EscapingBackslashAtEndOfInput) => panic!(),
            _ => {}
        }
    }
}
