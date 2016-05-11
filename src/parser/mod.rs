//! # Command Parser
//!
//! The ``commands::parser`` module provides:
//!
//! * Functionality for building a tree of commands based on their names.
//! * Parsing tokenized input to select a command. Input tokenization can
//!   be performed by the ``commands::tokenizer`` module.
//! * Validating parameters.
//! * Performing completion on commands and parameters.
//!
//! The parser is still a work in progress but is rapidly advancing.

#![allow(dead_code)]

pub mod nodes;
pub mod completion;

use std::collections::HashMap;
use std::rc::Rc;
use parser::nodes::*;
use tokenizer::{Token, TokenType};

/// Command parser
///
/// The lifetime parameter `'p` refers to the lifetime of the
/// root node of the command tree that was used to create this
/// parser.
///
/// The lifetime parameter `'p` refers to the lifetime of the parser
/// and of the tokens passed into this parser.
pub struct Parser<'p> {
    current_node: &'p Rc<Node>,
    /// The nodes which have been accepted during `parse` or `advance`.
    pub nodes: Vec<&'p Rc<Node>>,
    /// The tokens which have been accepted during `parse` or `advance`.
    pub tokens: Vec<&'p Token<'p>>,
    commands: Vec<Rc<CommandNode>>,
    parameters: HashMap<String, String>,
}

impl<'p> Parser<'p> {
    /// Construct a parser with a root node.
    pub fn new(initial_node: &'p Rc<Node>) -> Parser<'p> {
        Parser {
            current_node: initial_node,
            nodes: vec![],
            tokens: vec![],
            commands: vec![],
            parameters: HashMap::new(),
        }
    }

    /// XXX: Temporarily public.
    pub fn push_command(&mut self, command: Rc<CommandNode>) {
        self.commands.push(command);
    }

    /// Parse a vector of tokens, advancing through the
    /// node hierarchy.
    pub fn parse(&mut self, tokens: Vec<&'p Token<'p>>) {
        for token in tokens {
            if token.token_type != TokenType::Whitespace {
                self.advance(token);
            }
        }
    }

    /// Parse a single token, advancing through the node hierarchy.
    pub fn advance(&mut self, token: &'p Token<'p>) {
        unimplemented!();
    }

    /// Execute the command that has been accepted by the parser.
    ///
    /// * XXX: This isn't implemented yet.
    /// * XXX: This should be returning a Result probably.
    pub fn execute(&self) {
        if !self.commands.is_empty() {
            // self.commands[0].execute(self)
        }
    }

    /// Verify that the parser is in a valid state with
    /// respect to having accepted a command and all
    /// required parameters.
    ///
    /// * XXX: This isn't implemented yet.
    /// * XXX: This should be returning Option or Result
    ///        with an enum for the various error conditions.
    pub fn verify(&self) -> bool {
        if self.commands.is_empty() {
            // XXX: We'll want an enum error here with a Result.
            return false;
        } else {
            for expected in self.commands[0].parameters() {
                if expected.required() {
                    // if !self.parameters.contains_key(expected.name()) {
                    // return false;
                    // }
                }
            }
        }
        true
    }
}
