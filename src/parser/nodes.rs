//! # Parser Nodes
//!
//! Parser nodes represent the grammar that is defined
//! by the currently permissible set of commands and their
//! parameters.

#![allow(dead_code, missing_docs, unused_variables)]

/// Minimum priority.
pub const PRIORITY_MINIMUM: i32 = -10000;
/// The default priority for a parameter.
pub const PRIORITY_PARAMETER: i32 = -10;
/// The default priority.
pub const PRIORITY_DEFAULT: i32 = 0;

pub trait Node<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n>;

    fn successors(&'n self) -> &Vec<&'n Node<'n>> {
        &self.node_data().successors
    }

    fn help_symbol(&'n self) -> String {
        self.node_data().name.to_string()
    }

    fn help_text(&'n self) -> String {
        String::from("")
    }

    fn hidden(&'n self) -> bool {
        self.node_data().hidden
    }

    fn name(&'n self) -> &str {
        self.node_data().name
    }

    fn priority(&'n self) -> i32 {
        self.node_data().priority
    }
}

/// A parse tree node.
pub struct NodeFields<'n> {
    /// Possible successor nodes. Collected while building.
    pub successors: Vec<&'n Node<'n>>,
    /// The name of this node.
    pub name: &'n str,
    /// Match and complete priority.
    pub priority: i32,
    /// Hidden nodes are not completed. This doesn't modify matching.
    pub hidden: bool,
}

pub struct RootNode<'n> {
    fields: NodeFields<'n>,
}

impl<'n> Node<'n> for RootNode<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n> {
        &self.fields
    }
}

pub struct CommandNode<'n> {
    fields: CommandNodeFields<'n>,
}

pub struct CommandNodeFields<'n> {
    node: NodeFields<'n>,
    help: &'n str,
    handler: fn(&node: Node) -> (),
    parameters: Vec<&'n Node<'n>>,
}

impl<'n> Node<'n> for CommandNode<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n> {
        &self.fields.node
    }
}

pub struct WrapperNode<'n> {
    fields: WrapperNodeFields<'n>,
}

pub struct WrapperNodeFields<'n> {
    command: CommandNodeFields<'n>,
    root: &'n Node<'n>,
}

impl<'n> Node<'n> for WrapperNode<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n> {
        &self.fields.command.node
    }
}

pub struct ParameterNameNode<'n> {
    fields: ParameterNameNodeFields<'n>,
}

pub struct RepeatableNodeFields<'n> {
    repeatable: bool,
    repeat_marker: Option<&'n Node<'n>>,
}

pub struct ParameterNameNodeFields<'n> {
    node: NodeFields<'n>,
    repeatable: RepeatableNodeFields<'n>,
    help: &'n str,
    parameter: &'n Node<'n>,
}

impl<'n> Node<'n> for ParameterNameNode<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n> {
        &self.fields.node
    }

    fn help_symbol(&'n self) -> String {
        self.fields.node.name.to_string() + " " + self.fields.parameter.help_symbol().as_str()
    }

    fn help_text(&'n self) -> String {
        self.fields.help.to_string()
    }
}

impl<'n> ParameterNameNode<'n> {
    pub fn repeatable(&'n self) -> bool {
        self.fields.repeatable.repeatable
    }

    pub fn repeat_marker(&'n self) -> Option<&'n Node<'n>> {
        self.fields.repeatable.repeat_marker
    }
}

pub trait ParameterNode<'n> {
    fn parameter_data(&'n self) -> &'n ParameterNodeFields<'n>;

    fn repeatable(&'n self) -> bool {
        self.parameter_data().repeatable.repeatable
    }

    fn repeat_marker(&'n self) -> Option<&'n Node<'n>> {
        self.parameter_data().repeatable.repeat_marker
    }

    fn required(&'n self) -> bool {
        self.parameter_data().required
    }
}

pub struct ParameterNodeFields<'n> {
    node: NodeFields<'n>,
    repeatable: RepeatableNodeFields<'n>,
    help: &'n str,
    required: bool,
}

impl<'n> Node<'n> for ParameterNode<'n> {
    fn node_data(&'n self) -> &'n NodeFields<'n> {
        &self.parameter_data().node
    }

    fn help_symbol(&'n self) -> String {
        String::from("<") + self.node_data().name +
        if self.repeatable() {
            ">..."
        } else {
            ">"
        }
    }

    fn help_text(&'n self) -> String {
        self.parameter_data().help.to_string()
    }
}

pub struct FlagParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
}

impl<'n> ParameterNode<'n> for FlagParameterNode<'n> {
    fn parameter_data(&'n self) -> &'n ParameterNodeFields<'n> {
        &self.fields
    }
}

pub struct NamedParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
}

impl<'n> ParameterNode<'n> for NamedParameterNode<'n> {
    fn parameter_data(&'n self) -> &'n ParameterNodeFields<'n> {
        &self.fields
    }
}

pub struct SimpleParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
}

impl<'n> ParameterNode<'n> for SimpleParameterNode<'n> {
    fn parameter_data(&'n self) -> &'n ParameterNodeFields<'n> {
        &self.fields
    }
}
