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
/// The lifetime parameter `'p` refers to the lifetime of the
/// parser itself, the root node of the command tree that was
/// used to create this parser, as well as the tokens passed
/// into the parser.
///
/// ```
/// use commands::parser::nodes::RootNode;
/// use commands::parser::Parser;
///
/// let root = RootNode::new();
/// let mut parser = Parser::new(root);
/// ```
///
/// The parser is constructed as a `mut`able object as most of
/// the methods on it will modify its state.
pub struct Parser<'p> {
    current_node: Rc<Node>,
    /// The nodes which have been accepted during `parse` or `advance`.
    pub nodes: Vec<Rc<Node>>,
    /// The tokens which have been accepted during `parse` or `advance`.
    pub tokens: Vec<Token<'p>>,
    commands: Vec<Rc<CommandNode>>,
    parameters: HashMap<String, String>,
}

impl<'p> Parser<'p> {
    /// Construct a parser with a root node.
    pub fn new(initial_node: Rc<RootNode>) -> Parser<'p> {
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
    pub fn complete(&self, token: Option<Token<'p>>) -> Vec<Completion> {
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
    /// use commands::parser::nodes::RootNode;
    /// use commands::parser::Parser;
    /// use commands::tokenizer::tokenize;
    ///
    /// let root = RootNode::new();
    /// let mut parser = Parser::new(root);
    ///
    /// if let Ok(tokens) = tokenize("show interface") {
    ///     parser.parse(tokens);
    /// }
    /// ```
    pub fn parse(&mut self, tokens: Vec<Token<'p>>) -> Result<(), ParseError<'p>> {
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
    pub fn advance(&mut self, token: Token<'p>) -> Result<(), ParseError<'p>> {
        // We clone the current node so that it doesn't stay borrowed
        // and break things when we try to modify it below.
        let cn = self.current_node.clone();
        let matches = cn.successors()
                        .into_iter()
                        .filter(|n| n.acceptable(self) && n.matches(self, token))
                        .collect::<Vec<_>>();
        match matches.len() {
            1 => {
                let matching_node = matches[0];
                matching_node.accept(self, token);
                self.current_node = matching_node.clone();
                self.nodes.push(matching_node.clone());
                self.tokens.push(token);
                Ok(())
            }
            0 => Err(ParseError::NoMatches(token)),
            _ => Err(ParseError::AmbiguousMatch(token)),
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
#[derive(Clone,Debug)]
pub enum ParseError<'t> {
    /// The parser is in an invalid state.
    InvalidState,
    /// There were no matches for the token.
    NoMatches(Token<'t>),
    /// There was more than 1 possible match for the token.
    AmbiguousMatch(Token<'t>), // XXX: One day, add: Vec<&'p Rc<Node>>),
}

impl<'t> Error for ParseError<'t> {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidState => "Invalid state.",
            ParseError::NoMatches(_) => "No match.",
            ParseError::AmbiguousMatch(_) => "Ambiguous match.",
        }
    }
}

impl<'t> fmt::Display for ParseError<'t> {
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
    fn accept<'p>(&self, parser: &mut Parser<'p>, token: Token);
}

impl Accept for Node {
    /// By default, nothing needs to happen for `accept`.
    fn accept<'p>(&self, _parser: &mut Parser<'p>, _token: Token) {}
}

impl Accept for Rc<CommandNode> {
    /// Record this command.
    fn accept<'p>(&self, parser: &mut Parser<'p>, _token: Token) {
        if let Some(_) = self.handler() {
            parser.commands.push(self.clone())
        }
    }
}

impl Accept for ParameterNode {
    /// Record this parameter value.
    fn accept<'p>(&self, parser: &mut Parser<'p>, token: Token) {
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
    use super::*;

    #[test]
    #[should_panic]
    fn verify_signals_no_command() {
        let root = RootNode::new();
        let parser = Parser::new(root);
        match parser.verify() {
            Err(VerifyError::NoCommandAccepted) => panic!(),
            _ => {}
        }
    }
}
