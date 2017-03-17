// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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

/// Represents the result of completing a node. Each valid completion
/// is represented by a [`CompletionOption`].
///
/// This may be hinted by a pre-existing token.
///
/// If a completion is exhaustive, then only the [`CompletionOption`]s
/// provided are valid.
///
/// The lifetime parameter `'text` refers to the lifetime of the
/// body of text which generated the [`Token`].
///
/// [`CompletionOption`]: struct.CompletionOption.html
/// [`Token`]: ../tokenizer/struct.Token.html
pub struct Completion<'text> {
    /// Value placeholder for help.
    pub help_symbol: String,
    /// Main help text.
    pub help_text: String,
    /// Token used to hint the completion, if provided.
    pub token: Option<Token<'text>>,
    /// Was this completion exhaustive? If yes, then only
    /// the given completion options are valid.
    pub exhaustive: bool,
    /// The actual completion options.
    pub options: Vec<CompletionOption>,
}

impl<'text> Completion<'text> {
    /// Construct a new Completion.
    pub fn new(help_symbol: String,
               help_text: String,
               token: Option<Token<'text>>,
               exhaustive: bool,
               complete_options: Vec<&str>,
               other_options: Vec<&str>)
               -> Completion<'text> {
        // Preserve all of the options while still &str so that
        // we can use this with longest_common_prefix later.
        let mut all_options = complete_options.clone();
        all_options.extend(other_options.iter().cloned());

        // Convert to String...
        let mut complete_options =
            complete_options.iter().map(|o| o.to_string()).collect::<Vec<_>>();
        let mut other_options = other_options.iter().map(|o| o.to_string()).collect::<Vec<_>>();
        // Apply token restrictions
        if let Some(t) = token {
            // Filter options using token.
            let token_text = t.text.to_string();
            complete_options.retain(|o| o.starts_with(t.text));
            other_options.retain(|o| o.starts_with(t.text));
            // If not exhaustive, then add the current token as
            // an incomplete option.
            if !exhaustive && !complete_options.contains(&token_text) &&
               !other_options.contains(&token_text) {
                other_options.push(token_text);
            }
        }
        // Add longest common prefix as an incomplete options, but
        // filter it against the existing options and the token.
        let lcp = longest_common_prefix(all_options).to_string();
        if !complete_options.contains(&lcp) && !other_options.contains(&lcp) {
            match token {
                Some(t) => {
                    if lcp != t.text {
                        other_options.push(lcp)
                    }
                }
                None => other_options.push(lcp),
            }
        }
        // Convert options to CompletionOption.
        let mut options = complete_options.iter()
            .map(|o| CompletionOption::new(o.clone(), true))
            .collect::<Vec<_>>();
        options.extend(other_options.iter().map(|o| CompletionOption::new(o.clone(), false)));
        Completion {
            help_symbol: help_symbol,
            help_text: help_text,
            token: token,
            exhaustive: exhaustive,
            options: options,
        }
    }
}
