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
//! # Examples
//!
//! ```
//! use commands::tokenizer::tokenize;
//!
//! let tokens = tokenize("word");
//! assert_eq!(tokens.len(), 1);
//!
//! let tokens = tokenize("show interface");
//! assert_eq!(tokens.len(), 3);
//!
//! let tokens = tokenize("echo -n \"a b c\"");
//! assert_eq!(tokens.len(), 5);
//! ```

/// A position within a string.
///
/// The `SourceOffset` tracks 2 different ways of locating the
/// position:
///
/// * The index of the character within the string.
/// * The column and line number of the character.
///
/// The `SourceOffset` is typically used as a pair of offsets
/// indicating the start and end of a range of text as used
/// by the `SourceLocation`.
#[derive(Debug,PartialEq)]
pub struct SourceOffset {
    /// The index of this character within the string.
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

/// A range within a string.
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

/// The role that a token plays: `Whitespace` or `Word`.
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum TokenType {
    /// Internal usage only.
    Invalid,
    /// The token represents whitespace and not a word.
    Whitespace,
    /// The token represents a word within the string. This
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

    fn tokenize(&mut self) {
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
                    if !c.is_whitespace() {
                        self.shift(offset, State::Word);
                    } else {
                        panic!("Character not allowed here.");
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
                        panic!("Character not allowed here.");
                    };
                }
                State::Special => {
                    panic!("Special not yet implemented.");
                }
            }
        }

        // Now for the end of the text...
        match self.state {
            State::Initial => {}
            State::Word => self.reduce(),
            State::Whitespace => self.reduce(),
            State::WordBackslash => panic!("Escaping backslash at end of input"),
            State::Doublequote => panic!("Unclosed double quote at end of input"),
            State::DoublequoteBackslash => panic!("Escaped doublequote at end of input"),
            State::Special => panic!("Special not yet implemented"),
        }
    }
}

/// Tokenize a body of text.
pub fn tokenize(text: &str) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(text);
    tokenizer.tokenize();
    tokenizer.tokens
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
        let ts = tokenize("");
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn single_word() {
        let ts = tokenize("a");
        assert_eq!(ts.len(), 1);
        assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
    }

    #[test]
    fn multiple_words() {
        let ts = tokenize(" aa bb  ccc ");
        assert_eq!(ts.len(), 7);
        assert_eq!(ts[0], mk_token(" ", TokenType::Whitespace, 0, 0));
        assert_eq!(ts[1], mk_token("aa", TokenType::Word, 1, 2));
        assert_eq!(ts[2], mk_token(" ", TokenType::Whitespace, 3, 3));
        assert_eq!(ts[3], mk_token("bb", TokenType::Word, 4, 5));
        assert_eq!(ts[4], mk_token("  ", TokenType::Whitespace, 6, 7));
        assert_eq!(ts[5], mk_token("ccc", TokenType::Word, 8, 10));
        assert_eq!(ts[6], mk_token(" ", TokenType::Whitespace, 11, 11));
    }

    #[test]
    fn quoted_text() {
        let ts = tokenize("a \"b c\"");
        assert_eq!(ts.len(), 3);
        assert_eq!(ts[0], mk_token("a", TokenType::Word, 0, 0));
        assert_eq!(ts[1], mk_token(" ", TokenType::Whitespace, 1, 1));
        assert_eq!(ts[2], mk_token("\"b c\"", TokenType::Word, 2, 6));
    }
}
