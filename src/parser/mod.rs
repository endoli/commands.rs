// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Command Parser
//!
//! The `commands::parser` module provides:
//!
//! * Functionality for building a tree of commands based on their names.
//! * Parsing tokenized input to select a command. Input tokenization can
//!   be performed by the `commands::tokenizer` module.
//! * Validating parameters.
//! * Performing completion on commands and parameters.
//!
//! The command parser consists of two important things:
//!
//! * A tree that represents the available commands and their arguments.
//!   This tree consists of instances of `Nodes` from the
//!   `commands::parser::nodes` module. Construction of this tree
//!   is done with the help of the `commands::parser::builder` module.
//! * A `Parser` that handles input and matches it against the command
//!   tree. This parser is intended to be short-lived and to just live
//!   for the duration of parsing and evaluating a single command line
//!   input.

pub mod nodes;
pub mod completion;
pub mod builder;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use parser::nodes::*;
use tokenizer::{Token, TokenType};
use parser::completion::{Complete, Completion};

/// Command parser
///
/// The lifetime parameter `'text` refers to the lifetime of the
/// tokens passed into the parser. This is the same as the lifetime
/// of the text used to create the tokens.
///
/// When creating a `Parser`, you must give it an `Rc<RootNode>`.
/// The easiest and best way to get a root node is to use the
/// `commands::parser::builder` module.
///
/// ```
/// use commands::parser::builder::{Command, CommandTree};
/// use commands::parser::Parser;
///
/// let mut tree = CommandTree::new();
/// tree.command(Command::new("show"));
/// tree.command(Command::new("set"));
/// tree.command(Command::new("help"));
///
/// let mut parser = Parser::new(tree.finalize());
/// ```
///
/// The parser is constructed as a `mut`able object as most of
/// the methods on it will modify its state.
pub struct Parser<'text> {
    current_node: Rc<Node>,
    /// The nodes which have been accepted during `parse` or `advance`.
    pub nodes: Vec<Rc<Node>>,
    /// The tokens which have been accepted during `parse` or `advance`.
    pub tokens: Vec<Token<'text>>,
    commands: Vec<Rc<CommandNode>>,
    parameters: HashMap<String, String>,
}

impl<'text> Parser<'text> {
    /// Construct a parser with a root node.
    pub fn new(initial_node: Rc<RootNode>) -> Parser<'text> {
        Parser {
            current_node: initial_node,
            nodes: vec![],
            tokens: vec![],
            commands: vec![],
            parameters: HashMap::new(),
        }
    }

    /// Given an optional token, get the possible valid completions
    /// for the current parser state.
    ///
    /// Possible completions are successors of the current node which
    /// are not `hidden`, are `acceptable`, and which match the token,
    /// if one has been provided.
    ///
    /// Nodes may customize the `Complete` trait to customize the
    /// `Completion` and `CompletionOption`s which are generated
    /// for that node.
    ///
    /// Each valid successor node will have one `Completion` in the
    /// result vector. Each `Completion` will have one or more
    /// `CompletionOption` for each valid way that the value may be
    /// entered.
    ///
    /// ```
    /// use commands::parser::builder::{Command, CommandTree};
    /// use commands::parser::Parser;
    /// use commands::tokenizer::{Token, tokenize};
    ///
    /// let mut tree = CommandTree::new();
    /// tree.command(Command::new("show"));
    /// tree.command(Command::new("set"));
    /// tree.command(Command::new("help"));
    ///
    /// let mut parser = Parser::new(tree.finalize());
    ///
    /// // Completing now should have 3 options, 1 for each command.
    /// let comps = parser.complete(None);
    /// assert_eq!(comps.len(), 3);
    ///
    /// // But completing with a token for 'h' should have 1 option.
    /// if let Ok(tokens) = tokenize("h") {
    ///   let comps = parser.complete(Some(tokens[0]));
    ///   assert_eq!(comps.len(), 1);
    ///   assert_eq!(comps[0].options.len(), 1);
    ///   assert_eq!(comps[0].options[0].option_string, "help");
    /// } else {
    ///   panic!("Tokenize failed.");
    /// }
    ///
    /// // And completing for 's' should have 2 options.
    /// if let Ok(tokens) = tokenize("s") {
    ///   let comps = parser.complete(Some(tokens[0]));
    ///   assert_eq!(comps.len(), 2);
    /// } else {
    ///   panic!("Tokenize failed.");
    /// }
    /// ```
    pub fn complete(&self, token: Option<Token<'text>>) -> Vec<Completion> {
        self.current_node
            .successors()
            .into_iter()
            .filter(|n| {
                // To be a possible completion, the node should not be
                // hidden, it should be acceptable, and if there's a token,
                // it should be a valid match for the node.
                !n.hidden() && n.acceptable(self) &&
                if let Some(t) = token {
                    n.matches(self, t)
                } else {
                    true
                }
            })
            .map(|n| n.complete(token))
            .collect::<Vec<_>>()
    }

    /// Parse a vector of tokens, advancing through the
    /// node hierarchy.
    ///
    /// ```
    /// use commands::parser::builder::{Command, CommandTree};
    /// use commands::parser::Parser;
    /// use commands::tokenizer::tokenize;
    ///
    /// let mut tree = CommandTree::new();
    /// tree.command(Command::new("show interface"));
    ///
    /// let mut parser = Parser::new(tree.finalize());
    ///
    /// if let Ok(tokens) = tokenize("show interface") {
    ///     parser.parse(tokens);
    /// }
    /// ```
    pub fn parse(&mut self, tokens: Vec<Token<'text>>) -> Result<(), ParseError<'text>> {
        for token in tokens {
            match token.token_type {
                TokenType::Invalid => unreachable!(),
                TokenType::Whitespace => {}
                TokenType::Word => try!(self.advance(token)),
            }
        }
        Ok(())
    }

    /// Parse a single token, advancing through the node hierarchy.
    pub fn advance(&mut self, token: Token<'text>) -> Result<(), ParseError<'text>> {
        // We clone the current node so that it doesn't stay borrowed
        // and break things when we try to modify it below.
        let cn = self.current_node.clone();
        let acceptable = cn.successors()
                           .into_iter()
                           .filter(|n| n.acceptable(self))
                           .map(|n| n.clone())
                           .collect::<Vec<_>>();
        let matches = acceptable.clone()
                                .into_iter()
                                .filter(|n| n.matches(self, token))
                                .map(|n| n.clone())
                                .collect::<Vec<_>>();
        match matches.len() {
            1 => {
                let ref matching_node = matches[0];
                matching_node.accept(self, token);
                self.current_node = matching_node.clone();
                self.nodes.push(matching_node.clone());
                self.tokens.push(token);
                Ok(())
            }
            0 => Err(ParseError::NoMatches(token, acceptable)),
            _ => Err(ParseError::AmbiguousMatch(token, matches)),
        }
    }

    /// Execute the command that has been accepted by the parser.
    ///
    /// * XXX: This should be returning a Result probably.
    pub fn execute(&self) {
        if !self.commands.is_empty() {
            unimplemented!();
            // self.commands[0].execute(self)
        }
    }

    /// Verify that the parser is in a valid state with
    /// respect to having accepted a command and all
    /// required parameters.
    pub fn verify(&self) -> Result<(), VerifyError> {
        if self.commands.is_empty() {
            return Err(VerifyError::NoCommandAccepted);
        } else {
            for expected in self.commands[0].parameters() {
                if expected.required() && !self.parameters.contains_key(expected.name()) {
                    return Err(VerifyError::MissingParameter(expected.name().clone()));
                }
            }
        }
        Ok(())
    }
}

/// Errors that calling `parse` on the `Parser` can raise.
#[derive(Clone)]
pub enum ParseError<'text> {
    /// There were no matches for the token.
    NoMatches(Token<'text>, Vec<Rc<Node>>),
    /// There was more than 1 possible match for the token.
    AmbiguousMatch(Token<'text>, Vec<Rc<Node>>),
}

impl<'text> fmt::Debug for ParseError<'text> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::NoMatches(token, _) => write!(f, "NoMatches({:?}, ...)", token),
            ParseError::AmbiguousMatch(token, _) => write!(f, "AmbiguousMatch({:?}, ...)", token),
        }
    }
}

impl<'text> Error for ParseError<'text> {
    fn description(&self) -> &str {
        match *self {
            ParseError::NoMatches(_, _) => "No match.",
            ParseError::AmbiguousMatch(_, _) => "Ambiguous match.",
        }
    }
}

impl<'text> fmt::Display for ParseError<'text> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

/// Errors that calling `verify` on the `Parser` can raise.
#[derive(Clone,Debug)]
pub enum VerifyError {
    /// No command has been accepted by the parser.
    NoCommandAccepted,
    /// A required parameter is missing.
    MissingParameter(String),
}

impl Error for VerifyError {
    fn description(&self) -> &str {
        match *self {
            VerifyError::NoCommandAccepted => "No command has been accepted by the parser.",
            VerifyError::MissingParameter(_) => "A required parameter is missing.",
        }
    }
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

/// Can this node be accepted in the current parser state?
pub trait Acceptable {
    /// Can this node be accepted in the current parser state?
    fn acceptable(&self, parser: &Parser) -> bool;
}

impl Acceptable for Rc<Node> {
    /// By default, a node can be accepted when it hasn't been seen yet.
    fn acceptable(&self, parser: &Parser) -> bool {
        !parser.nodes.contains(self)
    }
}

impl Acceptable for RepeatableNode {
    /// A repeatable node can be accepted.
    fn acceptable(&self, _parser: &Parser) -> bool {
        if self.repeatable() {
            return true;
        }
        unimplemented!()
        // This should check nodes.contains, but then go on to check
        // for a repeat marker and whether or not that's been seen.
    }
}

/// Define whether or not a node matches a token.
pub trait Matches {
    /// Does this node match the specified `token`?
    fn matches(&self, parser: &Parser, token: Token) -> bool;
}

impl Matches for Node {
    /// By default, a node matches a `token` when the name of the
    /// node starts with the `token`.
    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.name().starts_with(token.text)
    }
}

impl Matches for ParameterNode {
    /// Parameters can match any token by default.
    fn matches(&self, _parser: &Parser, _token: Token) -> bool {
        true
    }
}

impl Matches for FlagParameterNode {
    /// A flag parameter is just looking for the token matching
    /// its name.
    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.name().starts_with(token.text)
    }
}

/// Define the behavior of a node when it is accepted by the parser.
pub trait Accept {
    /// Accept this node with the given `token` as data.
    ///
    /// This is where parameters are stored, commands added.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token);
}

impl Accept for Node {
    /// By default, nothing needs to happen for `accept`.
    fn accept<'text>(&self, _parser: &mut Parser<'text>, _token: Token) {}
}

impl Accept for Rc<CommandNode> {
    /// Record this command.
    fn accept<'text>(&self, parser: &mut Parser<'text>, _token: Token) {
        if self.handler().is_some() {
            parser.commands.push(self.clone())
        }
    }
}

impl Accept for ParameterNode {
    /// Record this parameter value.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }
}

#[cfg(test)]
mod test {
    use super::nodes::*;
    use super::builder::{Command, CommandTree};
    use super::*;
    use tokenizer::tokenize;

    #[test]
    #[should_panic]
    fn verify_signals_no_command() {
        let root = RootNode::new(vec![]);
        let parser = Parser::new(root);
        match parser.verify() {
            Err(VerifyError::NoCommandAccepted) => panic!(),
            _ => {}
        }
    }

    #[test]
    #[should_panic]
    fn parse_signals_no_matches() {
        let mut tree = CommandTree::new();
        tree.command(Command::new("show"));
        let mut parser = Parser::new(tree.finalize());
        if let Ok(tokens) = tokenize("h") {
            match parser.parse(tokens) {
                Err(ParseError::NoMatches(_, _)) => panic!(),
                _ => {}
            }
        }
    }

    #[test]
    #[should_panic]
    fn parse_signals_ambiguous_match() {
        let mut tree = CommandTree::new();
        tree.command(Command::new("show"));
        tree.command(Command::new("set"));
        let mut parser = Parser::new(tree.finalize());
        if let Ok(tokens) = tokenize("s") {
            match parser.parse(tokens) {
                Err(ParseError::AmbiguousMatch(_, _)) => panic!(),
                _ => {}
            }
        }
    }
}
