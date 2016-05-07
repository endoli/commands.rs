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
    fn as_command_node(&self) -> Option<CommandNode> {
        None
    }
}

pub struct RootNode<'n> {
    fields: NodeFields<'n>,
}

impl<'n> Node<'n> for RootNode<'n> {}

pub struct CommandNode<'n> {
    fields: CommandNodeFields<'n>,
}

impl<'n> Node<'n> for CommandNode<'n> {}

pub struct WrapperNode<'n> {
    fields: WrapperNodeFields<'n>,
}

impl<'n> Node<'n> for WrapperNode<'n> {}

pub struct ParameterNameNode<'n> {
    fields: ParameterNameNodeFields<'n>,
}

impl<'n> Node<'n> for ParameterNameNode<'n> {}

pub trait ParameterNode<'n> {}

pub struct FlagParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
}

pub struct NamedParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
}

pub struct SimpleParameterNode<'n> {
    fields: ParameterNodeFields<'n>,
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

pub struct CommandNodeFields<'n> {
    node: NodeFields<'n>,
    help: &'n str,
    handler: fn(&node: Node) -> (),
    parameters: Vec<&'n Node<'n>>,
}

pub struct WrapperNodeFields<'n> {
    command: CommandNodeFields<'n>,
    root: &'n Node<'n>,
}

pub struct RepeatableNodeFields<'n> {
    repeatable: bool,
    repeat_marker: Option<&'n Node<'n>>,
}

pub struct ParameterNameNodeFields<'n> {
    node: NodeFields<'n>,
    repeatable: RepeatableNodeFields<'n>,
    parameter: &'n Node<'n>,
}

pub struct ParameterNodeFields<'n> {
    node: NodeFields<'n>,
    repeatable: RepeatableNodeFields<'n>,
    help: &'n str,
    required: bool,
}
