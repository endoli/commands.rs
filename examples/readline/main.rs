// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate commands;
extern crate readline;

use commands::parser::builder::{Command, CommandTree};
use commands::parser::nodes::Node;
use commands::parser::{ParseError, Parser};
use commands::tokenizer::tokenize;
use readline::readline;

use std::ffi::CString;
use std::io::Write;
use std::io;

// This will be a lot nicer once readline 0.0.12 is published
// to crates.io and I can use it.

fn main() {
    let mut tree = CommandTree::new();
    tree.command(Command::new("show").finalize());
    let root = tree.finalize();

    let prompt = CString::new(">> ").unwrap();
    while let Ok(s) = readline(&prompt) {
        if let Ok(tokens) = tokenize(s.to_str().unwrap()) {
            let mut parser = Parser::new(root.clone());
            match parser.parse(tokens) {
                Ok(_) => {}
                Err(ParseError::NoMatches(_, acceptable)) => {
                    print!("\nPossible options:\n");
                    for ref option in acceptable {
                        print!("  {} - {}\n", option.help_symbol(), option.help_text());
                    }
                }
                Err(ParseError::AmbiguousMatch(_, matches)) => {
                    print!("\nCan be interpreted as:\n");
                    for ref option in matches {
                        print!("  {} - {}\n", option.help_symbol(), option.help_text());
                    }
                }
            }
        }
        io::stdout().write_all(b"\n").unwrap();
    }
    io::stdout().write_all(b"\nExiting.\n").unwrap();
}
