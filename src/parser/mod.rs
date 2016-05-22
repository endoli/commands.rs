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
//!   be performed by the [`commands::tokenizer`] module.
//! * Validating parameters.
//! * Performing completion on commands and parameters.
//!
//! The command parser consists of two important things:
//!
//! * A tree that represents the available commands and their arguments.
//!   This tree consists of instances of implementations of [`Node`] like
//!   [`CommandNode`], [`ParameterNode`] and [`RootNode`]. Construction of this
//!   tree is done with the help of [`CommandTree`], [`Command`] and
//!   [`Parameter`].
//! * A [`Parser`] that handles input and matches it against the command
//!   tree. This parser is intended to be short-lived and to just live
//!   for the duration of parsing and evaluating a single command line
//!   input.
//!
//! ## Building Nodes
//!
//! Building a tree of nodes for use with the parser is best done with
//! the [`CommandTree`] in conjunction with [`Command`] and [`Parameter`].
//!
//! Start by creating a mutable [`CommandTree`] instance:
//!
//! ```
//! use commands::parser::{CommandTree, Parser};
//!
//! let mut tree = CommandTree::new();
//! ```
//!
//! Then, add your commands and arguments, and finally,
//! call `finalize` on the tree to get back a [`RootNode`]
//! that can use be used with a [`Parser`].
//!
//! ```
//! use commands::parser::{Command, CommandTree, Parameter, Parser};
//!
//! let mut tree = CommandTree::new();
//! tree.command(Command::new("again")
//!                  .hidden(false)
//!                  .parameter(Parameter::new("test")
//!                                 .required(false)
//!                                 .help("This is just a test parameter.")));
//! let root = tree.finalize();
//! let mut parser = Parser::new(root);
//! ```
//!
//! [`commands::tokenizer`]: ../tokenizer/index.html
//! [`Command`]: struct.Command.html
//! [`CommandNode`]: struct.CommandNode.html
//! [`CommandTree`]: struct.CommandTree.html
//! [`Node`]: trait.Node.html
//! [`Parameter`]: struct.Parameter.html
//! [`ParameterNode`]: trait.ParameterNode.html
//! [`Parser`]: struct.Parser.html
//! [`RootNode`]: struct.RootNode.html

mod builder;
mod completion;
mod nodes;

// Re-export public API
pub use self::builder::{Command, CommandTree, Parameter};
pub use self::completion::{Completion, CompletionOption};
pub use self::nodes::{CommandNode, FlagParameterNode, NamedParameterNode, Node, NodeData,
                      ParameterNameNode, ParameterNode, RepeatableNode, RootNode,
                      SimpleParameterNode, WrapperNode};
pub use self::nodes::{PRIORITY_DEFAULT, PRIORITY_MINIMUM, PRIORITY_PARAMETER};

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use tokenizer::{Token, TokenType};

/// Command parser
///
/// The lifetime parameter `'text` refers to the lifetime of the
/// tokens passed into the parser. This is the same as the lifetime
/// of the text used to create the tokens.
///
/// When creating a `Parser`, you must give it an `Rc<RootNode>`.
/// [`RootNode`] instances should be created using a [`CommandTree`].
///
/// ```
/// use commands::parser::{Command, CommandTree, Parser};
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
///
/// [`CommandTree`]: struct.CommandTree.html
/// ['RootNode`]: struct.RootNode.html
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
    /// [`Completion`] and [`CompletionOption`]s which are generated
    /// for that node.
    ///
    /// Each valid successor node will have one [`Completion`] in the
    /// result vector. Each [`Completion`] will have one or more
    /// [`CompletionOption`] for each valid way that the value may be
    /// entered.
    ///
    /// ```
    /// use commands::parser::{Command, CommandTree, Parser};
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
    ///
    /// [`Completion`]: struct.Completion.html
    /// [`CompletionOption`]: struct.CompletionOption.html
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
    /// use commands::parser::{Command, CommandTree, Parser};
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
                let matching_node = &matches[0];
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

#[cfg(test)]
mod test {
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
