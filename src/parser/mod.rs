//! # Command Parser
//!
//! The ``commands::parser`` module provides:
//!
//! * Functionality for building a tree of commands based on their names.
//! * Tokenizing and parsing input to select a command.
//! * Validating parameters.
//! * Performing completion on commands and parameters.

#![allow(dead_code)]

pub mod tokenizer;
pub mod nodes;
pub mod completion;

use std::collections::HashMap;
use parser::tokenizer::{Token, TokenType};
use parser::nodes::*;

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
    commands: Vec<&'r Node<'r>>,
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
    pub fn push_command(&mut self, command: &'r Node<'r>) {
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
}
