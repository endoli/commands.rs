//! # Parser Nodes
//!
//! Parser nodes represent the grammar that is defined
//! by the currently permissible set of commands and their
//! parameters.

#![allow(missing_docs)]

use std::rc::Rc;

/// Minimum priority.
pub const PRIORITY_MINIMUM: i32 = -10000;
/// The default priority for a parameter.
pub const PRIORITY_PARAMETER: i32 = -10;
/// The default priority.
pub const PRIORITY_DEFAULT: i32 = 0;

pub trait Node {
    fn node_data(&self) -> &NodeFields;

    fn successors(&self) -> &Vec<Rc<Node>> {
        &self.node_data().successors
    }

    fn help_symbol(&self) -> String {
        self.node_data().name.to_string()
    }

    fn help_text(&self) -> &String {
        unimplemented!();
    }

    fn hidden(&self) -> bool {
        self.node_data().hidden
    }

    fn name(&self) -> &String {
        &self.node_data().name
    }

    fn priority(&self) -> i32 {
        self.node_data().priority
    }
}

/// A parse tree node.
pub struct NodeFields {
    /// Possible successor nodes. Collected while building.
    pub successors: Vec<Rc<Node>>,
    /// The name of this node.
    pub name: String,
    /// Match and complete priority.
    pub priority: i32,
    /// Hidden nodes are not completed. This doesn't modify matching.
    pub hidden: bool,
}

pub struct CommandNode {
    fields: CommandNodeFields,
}

pub struct CommandNodeFields {
    node: NodeFields,
    help: String,
    handler: Option<fn(&node: Node) -> ()>,
    parameters: Vec<Rc<ParameterNode>>,
}

impl Node for CommandNode {
    fn node_data(&self) -> &NodeFields {
        &self.fields.node
    }

    fn help_text(&self) -> &String {
        &self.fields.help
    }
}

impl CommandNode {
    pub fn handler(&self) -> Option<fn(&node: Node) -> ()> {
        self.fields.handler
    }

    /// Get the parameter nodes for this command
    pub fn parameters(&self) -> &Vec<Rc<ParameterNode>> {
        &self.fields.parameters
    }
}

pub struct WrapperNode {
    fields: WrapperNodeFields,
}

pub struct WrapperNodeFields {
    command: CommandNodeFields,
    root: Rc<Node>,
}

impl Node for WrapperNode {
    fn node_data(&self) -> &NodeFields {
        &self.fields.command.node
    }

    fn successors(&self) -> &Vec<Rc<Node>> {
        &self.fields.root.successors()
    }
}

pub struct ParameterNameNode {
    fields: ParameterNameNodeFields,
}

pub struct RepeatableNodeFields {
    repeatable: bool,
    repeat_marker: Option<Rc<Node>>,
}

pub struct ParameterNameNodeFields {
    node: NodeFields,
    repeatable: RepeatableNodeFields,
    help: String,
    parameter: Rc<Node>,
}

impl Node for ParameterNameNode {
    fn node_data(&self) -> &NodeFields {
        &self.fields.node
    }

    fn help_symbol(&self) -> String {
        self.fields.node.name.clone() + " " + self.fields.parameter.help_symbol().as_str()
    }

    fn help_text(&self) -> &String {
        &self.fields.help
    }
}

impl ParameterNameNode {
    pub fn repeatable(&self) -> bool {
        self.fields.repeatable.repeatable
    }

    pub fn repeat_marker(&self) -> &Option<Rc<Node>> {
        &self.fields.repeatable.repeat_marker
    }
}

pub trait ParameterNode {
    fn parameter_data(&self) -> &ParameterNodeFields;

    fn repeatable(&self) -> bool {
        self.parameter_data().repeatable.repeatable
    }

    fn repeat_marker(&self) -> &Option<Rc<Node>> {
        &self.parameter_data().repeatable.repeat_marker
    }

    fn required(&self) -> bool {
        self.parameter_data().required
    }
}

pub struct ParameterNodeFields {
    node: NodeFields,
    repeatable: RepeatableNodeFields,
    help: String,
    required: bool,
}

impl Node for ParameterNode {
    fn node_data(&self) -> &NodeFields {
        &self.parameter_data().node
    }

    fn help_symbol(&self) -> String {
        String::from("<") + self.node_data().name.as_str() +
        if self.repeatable() {
            ">..."
        } else {
            ">"
        }
    }

    fn help_text(&self) -> &String {
        &self.parameter_data().help
    }
}

pub struct FlagParameterNode {
    fields: ParameterNodeFields,
}

impl ParameterNode for FlagParameterNode {
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.fields
    }
}

pub struct NamedParameterNode {
    fields: ParameterNodeFields,
}

impl ParameterNode for NamedParameterNode {
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.fields
    }
}

pub struct SimpleParameterNode {
    fields: ParameterNodeFields,
}

impl ParameterNode for SimpleParameterNode {
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.fields
    }
}
