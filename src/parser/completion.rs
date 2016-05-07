//! # Completion
//!

#![allow(dead_code, missing_docs, unused_variables)]

use parser::nodes::*;
use tokenizer::Token;

/// Represents a single option returned by `complete`.
///
/// An option may be `complete`, which means that it represents
/// a syntactically complete parameter value which can be used
/// as-is, whereas incomplete options are not valid values.
///
/// The lifetime parameter `'c` refers to the lifetime of the
/// owning `Completion`.
pub struct CompletionOption<'c> {
    /// String for this option.
    pub option_string: &'c str,
    /// True if this option is complete and a valid value.
    pub complete: bool,
}

impl<'c> CompletionOption<'c> {
    /// CompletionOption constructor
    pub fn new(option_string: &'c str, complete: bool) -> CompletionOption {
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
/// The lifetime parameter `'c` refers to the lifetime of
/// this `Completion`. The lifetime parameter `'t` refers to
/// the lifetime of the body of text which generated the
/// `Token`. The lifetime parameter `'n` refers to the lifetime
/// of the node which generated the completion.
pub struct Completion<'c, 'n, 't> {
    /// Value placeholder for help.
    pub help_symbol: &'n str,
    /// Main help text.
    pub help_text: &'n str,
    /// Token used to hint the completion, if provided.
    pub token: Option<&'t Token<'t>>,
    /// Was this completion exhaustive? If yes, then only
    /// the given completion options are valid.
    pub exhaustive: bool,
    /// The actual completion options.
    pub options: Vec<CompletionOption<'c>>,
}

impl<'c, 'n, 't> Completion<'c, 'n, 't> {
    pub fn new(node: &'n Node<'n>,
               token: Option<&'t Token<'t>>,
               exhaustive: bool,
               complete_options: Vec<&str>,
               other_options: Vec<&str>)
               -> Completion<'c, 'n, 't> {
        Completion {
            help_symbol: "<...>",
            help_text: "No help.",
            token: token,
            exhaustive: exhaustive,
            options: vec![],
        }
    }
}

/// Trait for nodes that support completion.
pub trait Complete<'c, 'n, 't> {
    /// Given a node and an optional token, provide the completion options.
    fn complete(&'n self, token: Option<&'t Token<'t>>) -> Completion<'c, 'n, 't>;
}

impl<'c, 'n, 't> Complete<'c, 'n, 't> for Node<'n> {
    fn complete(&'n self, token: Option<&'t Token<'t>>) -> Completion<'c, 'n, 't> {
        Completion::new(self, token, true, vec![], vec![])
    }
}

impl<'c, 'n, 't> Complete<'c, 'n, 't> for RootNode<'n> {
    fn complete(&'n self, token: Option<&'t Token<'t>>) -> Completion<'c, 'n, 't> {
        panic!("BUG: Can not complete a root node.");
    }
}
