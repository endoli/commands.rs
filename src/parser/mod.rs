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
use parser::nodes::*;
use tokenizer::{Token, TokenType};

/// Command parser
///
/// The lifetime parameter `'r` refers to the lifetime of the
/// root node of the command tree that was used to create this
/// parser.
///
/// The lifetime parameter `'p` refers to the lifetime of the parser
/// and of the tokens passed into this parser.
pub struct Parser<'r, 'p> {
    current_node: &'r Node<'r>,
    /// The nodes which have been accepted during `parse` or `advance`.
    pub nodes: Vec<&'r Node<'r>>,
    /// The tokens which have been accepted during `parse` or `advance`.
    pub tokens: Vec<&'p Token<'p>>,
    commands: Vec<&'r CommandNode<'r>>,
    parameters: HashMap<String, String>,
}

impl<'r, 'p> Parser<'r, 'p> {
    /// Construct a parser with a root node.
    pub fn new(initial_node: &'r RootNode<'r>) -> Parser<'r, 'p> {
        Parser {
            current_node: initial_node,
            nodes: vec![],
            tokens: vec![],
            commands: vec![],
            parameters: HashMap::new(),
        }
    }

    fn push_node(&mut self, token: &'p Token, node: &'r Node<'r>) {
        self.current_node = node;
        self.nodes.push(node);
        self.tokens.push(token);
    }

    /// XXX: Temporarily public.
    pub fn push_command(&mut self, command: &'r CommandNode<'r>) {
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
        self.push_node(token, self.current_node);
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
