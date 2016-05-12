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
/// parser itself, the root node of the command tree that was
/// used to create this parser, as well as the tokens passed
/// into the parser.
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
    pub fn new(initial_node: Rc<Node>) -> Parser<'p> {
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
    pub fn parse(&mut self, tokens: Vec<Token<'p>>) {
        for token in tokens {
            match token.token_type {
                TokenType::Invalid => unreachable!(),
                TokenType::Whitespace => {}
                TokenType::Word => self.advance(token),
            }
        }
    }

    /// Parse a single token, advancing through the node hierarchy.
    pub fn advance(&mut self, token: Token<'p>) {
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
                if expected.required() && !self.parameters.contains_key(expected.name()) {
                    return false;
                }
            }
        }
        true
    }
}

trait Acceptable {
    fn acceptable(&self, parser: &Parser) -> bool;
}

impl Acceptable for Rc<Node> {
    fn acceptable(&self, parser: &Parser) -> bool {
        !parser.nodes.contains(self)
    }
}

impl Acceptable for RepeatableNode {
    fn acceptable(&self, _parser: &Parser) -> bool {
        if self.repeatable() {
            return true;
        }
        unimplemented!()
        // This should check nodes.contains, but then go on to check
        // for a repeat marker and whether or not that's been seen.
    }
}

trait Matches {
    fn matches(&self, parser: &Parser, token: Token) -> bool;
}

impl Matches for Node {
    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.name().starts_with(token.text)
    }
}

trait Accept {
    fn accept<'p>(&self, parser: &mut Parser<'p>, token: Token);
}

impl Accept for Node {
    fn accept<'p>(&self, _parser: &mut Parser<'p>, _token: Token) {}
}

impl Accept for Rc<CommandNode> {
    fn accept<'p>(&self, parser: &mut Parser<'p>, _token: Token) {
        if let Some(_) = self.handler() {
            parser.commands.push(self.clone())
        }
    }
}

impl Accept for Rc<ParameterNode> {
    fn accept<'p>(&self, parser: &mut Parser<'p>, token: Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }
}
