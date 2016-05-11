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

pub mod nodes;
pub mod completion;

use std::collections::HashMap;
use std::rc::Rc;
use parser::nodes::*;
use tokenizer::{Token, TokenType};
use parser::completion::{Complete, Completion};

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
    commands: Vec<&'p Rc<CommandNode>>,
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

    /// Given an optional token, get the possible valid completions
    /// for the current parser state.
    pub fn complete(&self, token: Option<&'p Token<'p>>) -> Vec<Completion> {
        fn possible_completion(node: &Rc<Node>, parser: &Parser, token: Option<&Token>) -> bool {
            // To be a possible completion, the node should not be
            // hidden, it should be acceptable, and if there's a token,
            // it should be a valid match for the node.
            !node.hidden() && node.acceptable(parser) &&
            match token {
                Some(t) => node.matches(parser, t),
                _ => true,
            }
        }
        self.current_node
            .successors()
            .into_iter()
            .filter(|n| possible_completion(n, self, token))
            .map(|n| n.complete(token))
            .collect::<Vec<_>>()
    }

    /// Parse a vector of tokens, advancing through the
    /// node hierarchy.
    pub fn parse(&mut self, tokens: Vec<&'p Token<'p>>) {
        for token in tokens {
            match token.token_type {
                TokenType::Invalid => unreachable!(),
                TokenType::Whitespace => {}
                TokenType::Word => self.advance(token),
            }
        }
    }

    /// Parse a single token, advancing through the node hierarchy.
    pub fn advance(&mut self, token: &'p Token<'p>) {
        let matches = self.current_node
                          .successors()
                          .into_iter()
                          .filter(|n| n.acceptable(self) && n.matches(self, token))
                          .collect::<Vec<_>>();
        match matches.len() {
            1 => {
                let matching_node = &matches[0];
                matching_node.accept(self, token);
                self.current_node = matching_node.clone();
                self.nodes.push(matching_node);
                self.tokens.push(token);
            }
            0 => panic!("No matches for '{}'.", token.text),
            _ => panic!("Ambiguous matches for '{}'.", token.text),
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
    ///
    /// * XXX: This should be returning Option or Result
    ///        with an enum for the various error conditions.
    pub fn verify(&self) -> bool {
        if self.commands.is_empty() {
            // XXX: We'll want an enum error here with a Result.
            return false;
        } else {
            for expected in self.commands[0].parameters() {
                if expected.required() {
                    if !self.parameters.contains_key(expected.name()) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

trait Advance {
    fn acceptable(&self, parser: &Parser) -> bool;

    fn matches(&self, parser: &Parser, token: &Token) -> bool;
}

impl Advance for Node {
    fn acceptable(&self, _parser: &Parser) -> bool {
        unimplemented!();
        // !parser.nodes.contains(self)
    }

    fn matches(&self, _parser: &Parser, token: &Token) -> bool {
        self.name().starts_with(token.text)
    }
}

trait Accept {
    fn accept<'p>(&'p self, parser: &mut Parser<'p>, token: &Token);
}

impl Accept for Node {
    fn accept<'p>(&'p self, _parser: &mut Parser<'p>, _token: &Token) {}
}

impl Accept for Rc<CommandNode> {
    fn accept<'p>(&'p self, parser: &mut Parser<'p>, _token: &Token) {
        match self.handler() {
            Some(_) => parser.commands.push(self),
            _ => {}
        }
    }
}

impl Accept for Rc<ParameterNode> {
    fn accept<'p>(&'p self, parser: &mut Parser<'p>, token: &Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }
}
