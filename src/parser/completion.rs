//! # Completion
//!

#![allow(dead_code, missing_docs, unused_variables)]

use parser::nodes::*;
use tokenizer::Token;
use util::longest_common_prefix;

/// Represents a single option returned by `complete`.
///
/// An option may be `complete`, which means that it represents
/// a syntactically complete parameter value which can be used
/// as-is, whereas incomplete options are not valid values.
pub struct CompletionOption {
    /// String for this option.
    pub option_string: String,
    /// True if this option is complete and a valid value.
    pub complete: bool,
}

impl CompletionOption {
    /// CompletionOption constructor
    pub fn new(option_string: String, complete: bool) -> CompletionOption {
        CompletionOption {
            option_string: option_string,
            complete: complete,
        }
    }
}

/// Represents the result of completing a node.
///
/// This may be hinted by a pre-existing token.
///
/// If a completion is exhaustive, then only the `CompletionOption`s
/// provided are valid.
///
/// The lifetime parameter `'t` refers to the lifetime of the
/// body of text which generated the `Token`.
pub struct Completion<'t> {
    /// Value placeholder for help.
    pub help_symbol: String,
    /// Main help text.
    pub help_text: String,
    /// Token used to hint the completion, if provided.
    pub token: Option<&'t Token<'t>>,
    /// Was this completion exhaustive? If yes, then only
    /// the given completion options are valid.
    pub exhaustive: bool,
    /// The actual completion options.
    pub options: Vec<CompletionOption>,
}

impl<'t> Completion<'t> {
    pub fn new(help_symbol: String,
               help_text: String,
               token: Option<&'t Token<'t>>,
               exhaustive: bool,
               complete_options: Vec<&str>,
               other_options: Vec<&str>)
               -> Completion<'t> {
        Completion {
            help_symbol: help_symbol,
            help_text: help_text,
            token: token,
            exhaustive: exhaustive,
            options: vec![],
        }
    }
}

/// Trait for nodes that support completion.
pub trait Complete<'t> {
    /// Given a node and an optional token, provide the completion options.
    fn complete(&self, token: Option<&'t Token<'t>>) -> Completion<'t>;
}

impl<'t> Complete<'t> for Node {
    fn complete(&self, token: Option<&'t Token<'t>>) -> Completion<'t> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![],
                        vec![])
    }
}

impl<'t> Complete<'t> for CommandNode {
    fn complete(&self, token: Option<&'t Token<'t>>) -> Completion<'t> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![self.name()],
                        vec![])
    }
}

impl<'t> Complete<'t> for ParameterNameNode {
    fn complete(&self, token: Option<&'t Token<'t>>) -> Completion<'t> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![self.name()],
                        vec![])
    }
}
