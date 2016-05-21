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

fn main() {
    let mut tree = CommandTree::new();
    tree.command(Command::new("show").finalize());
    let root = tree.finalize();

    while let Ok(s) = readline(">> ") {
        if let Ok(tokens) = tokenize(&*s) {
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
        print!("\n");
    }
    print!("\nExiting.\n");
}
