// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Command Tables

use crate::menu_definition::MenuDefinition;
use std::rc::Rc;

/// A command
pub trait Command {
    /// Execute the command
    fn execute(&self);
}

/// Information about a command in a command table.
pub struct CommandTableEntry {
    /// Name of a command. This is used for command line entry.
    pub name: String,
    /// The underlying command that can be executed.
    pub command: Rc<dyn Command>,
}

/// A group of related commands
///
/// Command tables are a way of grouping a set of related
/// command objects and providing user interfaces to the
/// commands.
///
/// Command tables can be interacted with via menus,
/// toolbars, keystrokes, gestures, and more.
pub struct CommandTable {
    /// The name of the command table
    pub name: String,
    /// Tables inherited by this table
    pub inherit: Vec<Rc<CommandTable>>,
    /// Commands in this table
    pub commands: Vec<CommandTableEntry>,
    /// Menu description
    pub menu_definition: Option<MenuDefinition>,
}

impl CommandTable {
    /// Construct a `CommandTable`.
    pub fn new(
        name: String,
        inherit: Vec<Rc<CommandTable>>,
        commands: Vec<CommandTableEntry>,
    ) -> Rc<CommandTable> {
        Rc::new(CommandTable {
            name,
            inherit,
            commands,
            menu_definition: None,
        })
    }
}
