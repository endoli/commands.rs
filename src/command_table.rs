//! # Command Tables

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
    pub command: Rc<Command>,
}

/// Types of menu items.
pub enum MenuItem {
    /// A separator item in a menu.
    Separator,
}

/// The definition to be used to generate a menu or
/// toolbar in whatever GUI framework is being used.
pub struct MenuDefinition {
    /// The items in the menu. Ordered vector.
    pub items: Vec<MenuItem>,
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
    pub fn new(name: String,
               inherit: Vec<Rc<CommandTable>>,
               commands: Vec<CommandTableEntry>)
               -> Rc<CommandTable> {
        Rc::new(CommandTable {
            name: name,
            inherit: inherit,
            commands: commands,
            menu_definition: None,
        })
    }
}
