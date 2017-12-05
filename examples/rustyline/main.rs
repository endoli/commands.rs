// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate commands;
extern crate rustyline;

use commands::parser::{Command, CommandTree, Node, ParseError, Parser};
use commands::tokenizer::tokenize;
use rustyline::{Editor, Result};
use rustyline::completion::Completer;
use std::rc::Rc;

struct CommandCompleter {
    root: Rc<Node>,
}

impl CommandCompleter {
    fn new(root: Rc<Node>) -> Self {
        CommandCompleter { root: root }
    }
}

impl Completer for CommandCompleter {
    fn complete(&self, line: &str, _pos: usize) -> Result<(usize, Vec<String>)> {
        // TODO: This is an initial implementation that needs a lot more work.
        if let Ok(tokens) = tokenize(line) {
            let p = Parser::new(Rc::clone(&self.root));
            let cs = p.complete(Some(tokens[0]));
            if !cs.is_empty() {
                Ok((
                    0,
                    cs[0]
                        .options
                        .iter()
                        .map(|co| co.option_string.clone())
                        .collect(),
                ))
            } else {
                Ok((0, Vec::new()))
            }
        } else {
            Ok((0, Vec::new()))
        }
    }
}

fn main() {
    let mut tree = CommandTree::new();
    tree.command(Command::new("show"));
    let root = tree.finalize();

    let c = CommandCompleter::new(Rc::clone(&root));
    let mut rl = Editor::<CommandCompleter>::new();
    rl.set_completer(Some(c));
    while let Ok(line) = rl.readline(">> ") {
        rl.add_history_entry(&line);
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
