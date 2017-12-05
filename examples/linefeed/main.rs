// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate commands;
extern crate linefeed;

use std::rc::Rc;

use commands::parser::{Command, CommandTree, ParseError, Parser};
use commands::tokenizer::tokenize;
use linefeed::{Reader, ReadResult};

fn main() {
    let mut tree = CommandTree::new();
    tree.command(Command::new("show"));
    let root = tree.finalize();

    let mut reader = Reader::new("example").unwrap();
    reader.set_prompt(">> ");
    while let Ok(ReadResult::Input(line)) = reader.read_line() {
        reader.add_history(line.clone());
        if let Ok(tokens) = tokenize(&line) {
            let mut parser = Parser::new(Rc::clone(&root));
            if let Err(err) = parser.parse(tokens) {
                match err {
                    ParseError::NoMatches(_, acceptable) => {
                        println!("No match for '{}'", line);
                        println!("\nPossible options:");
                        for option in &acceptable {
                            let n = option.node();
                            println!("  {} - {}", n.help_symbol, n.help_text);
                        }
                    }
                    ParseError::AmbiguousMatch(_, matches) => {
                        println!("\nCan be interpreted as:");
                        for option in &matches {
                            let n = option.node();
                            println!("  {} - {}", n.help_symbol, n.help_text);
                        }
                    }
                }
            } else if let Err(err) = parser.verify() {
                println!("{}", err);
            } else {
                parser.execute();
            }
        }
        println!();
    }
    println!("\nExiting.");
}
