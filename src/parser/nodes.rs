// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Parser nodes represent the grammar that is defined
// by the currently permissible set of commands and their
// parameters.

use std::rc::Rc;

use parser::Completion;
use parser::Parser;
use tokenizer::Token;

/// Minimum priority.
pub const PRIORITY_MINIMUM: i32 = -10000;
/// The default priority for a parameter.
pub const PRIORITY_PARAMETER: i32 = -10;
/// The default priority.
pub const PRIORITY_DEFAULT: i32 = 0;

/// Access the data for a node in the tree of commands and
/// their parameters used by the [`Parser`].
///
/// [`Parser`]: struct.Parser.html
pub trait NodeData {
    /// The data describing this node.
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields;

    /// The text used to identify this node in help text.
    /// This is typically the node name, either in plain
    /// form or decorated for parameters.
    fn help_symbol(&self) -> &String {
        &self.node_data().help_symbol
    }

    /// Help text describing this node.
    fn help_text(&self) -> &String {
        &self.node_data().help_text
    }

    /// Hidden nodes are still found for matching, but are
    /// hidden from completion.
    fn hidden(&self) -> bool {
        self.node_data().hidden
    }

    /// The name of this node.
    fn name(&self) -> &String {
        &self.node_data().name
    }

    /// This priority of this node during matching and completion.
    fn priority(&self) -> i32 {
        self.node_data().priority
    }

    /// Nodes that are children of this node. Used by the
    /// [`Parser`] during `advance`, `complete`, etc.
    ///
    /// [`Parser`]: struct.Parser.html
    fn successors(&self) -> &Vec<Rc<Node>> {
        &self.node_data().successors
    }
}

/// The node in the tree of commands and parameters used in the
/// parser.
///
/// This trait defines the core operations which a node must
/// support.
pub trait Node: NodeData {
    /// Accept this node with the given `token` as data.
    ///
    /// By default, nothing needs to happen for `accept`.
    fn accept<'text>(&self, _parser: &mut Parser<'text>, _token: Token) {}

    /// Can this node be accepted in the current parser state?
    /// By default, a node can be accepted when it hasn't been seen yet.
    fn acceptable(&self, _parser: &Parser) -> bool {
        true
        // !parser.nodes.contains(self)
    }

    /// Given a node and an optional token, provide the completion options.
    ///
    /// By default, completion should complete for the name of the given
    /// node.
    ///
    /// This is the expected behavior for [`CommandNode`] as well as
    /// [`ParameterNameNode`].
    ///
    /// [`CommandNode`]: struct.CommandNode.html
    /// [`ParameterNameNode`]: struct.ParameterNameNode.html
    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![self.name()],
                        vec![])
    }

    /// By default, a node matches a `token` when the name of the
    /// node starts with the `token`.
    ///
    /// This is the desired behavior for ...
    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.name().starts_with(token.text)
    }
}

impl PartialEq for Node {
    /// Nodes are equal based on pointer equality.
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

/// A parse tree node.
#[doc(hidden)]
pub struct NodeFields {
    /// The name of this node.
    name: String,
    /// The text used to identify this node in help text.
    /// This is typically the node name, either in plain
    /// form or decorated for parameters.
    help_symbol: String,
    /// Help text describing this node.
    help_text: String,
    /// Hidden nodes are not completed. This doesn't modify matching.
    hidden: bool,
    /// Match and complete priority.
    priority: i32,
    /// Possible successor nodes. Collected while building.
    successors: Vec<Rc<Node>>,
}

/// The root of a command tree.
///
/// ```
/// use commands::parser::RootNode;
///
/// let root = RootNode::new(vec![]);
/// ```
pub struct RootNode {
    node_fields: NodeFields,
}

impl RootNode {
    /// Create a new `RootNode`
    pub fn new(successors: Vec<Rc<Node>>) -> Rc<Self> {
        Rc::new(RootNode {
            node_fields: NodeFields {
                name: "__root__".to_string(),
                help_symbol: "".to_string(),
                help_text: "".to_string(),
                hidden: false,
                priority: PRIORITY_DEFAULT,
                successors: successors,
            },
        })
    }
}

impl NodeData for RootNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for RootNode {
    /// A `RootNode` can not be completed.
    fn complete<'text>(&self, _token: Option<Token<'text>>) -> Completion<'text> {
        panic!("BUG: Can not complete a root node.");
    }
}

/// A node representing a command. Constructed via [`Command`] and [`CommandTree`].
///
/// [`Command`]: struct.Command.html
/// [`CommandNode`]: struct.CommandNode.html
pub struct CommandNode {
    node_fields: NodeFields,
    command_fields: CommandNodeFields,
}

struct CommandNodeFields {
    handler: Option<fn(&node: Node) -> ()>,
    parameters: Vec<Rc<ParameterNode>>,
}

impl CommandNode {
    /// Construct a new `CommandNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               handler: Option<fn(&node: Node) -> ()>,
               parameters: Vec<Rc<ParameterNode>>)
               -> Self {
        CommandNode {
            node_fields: NodeFields {
                name: name.to_string(),
                help_symbol: name.to_string(),
                help_text: help_text.unwrap_or("Command").to_string(),
                hidden: hidden,
                priority: priority,
                successors: successors,
            },
            command_fields: CommandNodeFields {
                handler: handler,
                parameters: parameters,
            },
        }
    }

    /// The handler which is executed once this node has been accepted.
    pub fn handler(&self) -> Option<fn(&node: Node) -> ()> {
        self.command_fields.handler
    }

    /// Get the parameter nodes for this command.
    pub fn parameters(&self) -> &Vec<Rc<ParameterNode>> {
        &self.command_fields.parameters
    }
}

impl NodeData for CommandNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for CommandNode {
    /// Record this command.
    fn accept<'text>(&self, _parser: &mut Parser<'text>, _token: Token) {
        if self.handler().is_some() {
            unimplemented!();
            // parser.commands.push(Rc::new(self))
        }
    }

    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        println!("MATCHES COMMAND!!!!");
        self.name().starts_with(token.text)
    }
}

/// A wrapper node wraps another command.
///
/// This is used for the help command so that it can complete
/// normal commands.
///
/// The `successors` will be those of the wrapped node.
pub struct WrapperNode {
    node_fields: NodeFields,
    #[allow(dead_code)]
    command_fields: CommandNodeFields,
    root: Rc<Node>,
}

impl NodeData for WrapperNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }

    fn successors(&self) -> &Vec<Rc<Node>> {
        self.root.successors()
    }
}

/// A repeatable node is an internal helper for representing
/// nodes that can be repeated, like some parameters.
pub trait RepeatableNode: Node {
    /// Internal data for a repeatable node.
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields;

    /// Whether or not this node can be repeated. A repeated
    /// node can be `accept`ed multiple times.
    fn repeatable(&self) -> bool {
        self.repeatable_data().repeatable
    }

    /// If present, this node will no longer be `acceptable`.
    fn repeat_marker(&self) -> &Option<Rc<Node>> {
        &self.repeatable_data().repeat_marker
    }

    /// A repeatable node can be accepted.
    fn acceptable(&self, _parser: &Parser) -> bool {
        if self.repeatable() {
            return true;
        }
        unimplemented!()
        // This should check nodes.contains, but then go on to check
        // for a repeat marker and whether or not that's been seen.
    }
}

/// The data for a repeatable node.
#[doc(hidden)]
pub struct RepeatableNodeFields {
    repeatable: bool,
    repeat_marker: Option<Rc<Node>>,
}

/// A node that represented the name portion of a named
/// parameter.
pub struct ParameterNameNode {
    node_fields: NodeFields,
    repeatable_fields: RepeatableNodeFields,
    /// The `parameter` named by this node.
    pub parameter: Rc<ParameterNode>,
}

impl ParameterNameNode {
    /// Construct a new `ParameterNameNode`.
    pub fn new(name: &str,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               parameter: Rc<ParameterNode>)
               -> Self {
        let help_text = parameter.help_text().clone();
        let help_symbol = name.to_string() + " " + parameter.help_symbol().as_str();
        ParameterNameNode {
            node_fields: NodeFields {
                name: name.to_string(),
                help_symbol: help_symbol,
                help_text: help_text,
                hidden: hidden,
                priority: priority,
                successors: successors,
            },
            repeatable_fields: RepeatableNodeFields {
                repeatable: repeatable,
                repeat_marker: repeat_marker,
            },
            parameter: parameter,
        }
    }
}

impl NodeData for ParameterNameNode {
    #[doc(hidden)]
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for ParameterNameNode {}

impl RepeatableNode for ParameterNameNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.repeatable_fields
    }
}

/// Parameter nodes.
pub trait ParameterNode: Node + RepeatableNode {
    /// Internal data for a parameter node.
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields;

    /// A `required` parameter must be supplied for the
    /// command line being parsed to be valid.
    fn required(&self) -> bool {
        self.parameter_data().required
    }
}

/// Data for parameter nodes.
#[doc(hidden)]
pub struct ParameterNodeFields {
    required: bool,
}

/// A flag parameter node. When present, this has a value of `true`.
pub struct FlagParameterNode {
    node_fields: NodeFields,
    repeatable_fields: RepeatableNodeFields,
    parameter_fields: ParameterNodeFields,
}

impl NodeData for FlagParameterNode {
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for FlagParameterNode {
    /// Record this parameter value.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }
}

impl RepeatableNode for FlagParameterNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.repeatable_fields
    }
}

impl ParameterNode for FlagParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}

impl FlagParameterNode {
    /// Construct a new `FlagParameterNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               required: bool)
               -> Self {
        let help_symbol = String::from("<") + name +
                          if repeatable {
            ">..."
        } else {
            ">"
        };
        FlagParameterNode {
            node_fields: NodeFields {
                name: name.to_string(),
                help_symbol: help_symbol,
                help_text: help_text.unwrap_or("Flag").to_string(),
                hidden: hidden,
                priority: priority,
                successors: successors,
            },
            repeatable_fields: RepeatableNodeFields {
                repeatable: repeatable,
                repeat_marker: repeat_marker,
            },
            parameter_fields: ParameterNodeFields { required: required },
        }
    }
}

/// A named parameter node. This has both a name and a value in the command line.
pub struct NamedParameterNode {
    node_fields: NodeFields,
    repeatable_fields: RepeatableNodeFields,
    parameter_fields: ParameterNodeFields,
}

impl NodeData for NamedParameterNode {
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for NamedParameterNode {
    /// Record this parameter value.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }

    /// By default a `NamedParameterNode` completes only to itself.
    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![],
                        vec![])
    }

    /// Named parameters can match any token by default.
    fn matches(&self, _parser: &Parser, _token: Token) -> bool {
        true
    }
}

impl RepeatableNode for NamedParameterNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.repeatable_fields
    }
}

impl ParameterNode for NamedParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}

impl NamedParameterNode {
    /// Construct a new `NamedParameterNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               required: bool)
               -> Self {
        NamedParameterNode {
            node_fields: NodeFields {
                name: name.to_string(),
                help_symbol: name.to_string(),
                help_text: help_text.unwrap_or("Parameter").to_string(),
                hidden: hidden,
                priority: priority,
                successors: successors,
            },
            repeatable_fields: RepeatableNodeFields {
                repeatable: repeatable,
                repeat_marker: repeat_marker,
            },
            parameter_fields: ParameterNodeFields { required: required },
        }
    }
}

/// A simple parameter node. This is only present in a command
/// line as a value.
pub struct SimpleParameterNode {
    node_fields: NodeFields,
    repeatable_fields: RepeatableNodeFields,
    parameter_fields: ParameterNodeFields,
}

impl NodeData for SimpleParameterNode {
    fn node_data(&self) -> &NodeFields {
        &self.node_fields
    }
}

impl Node for SimpleParameterNode {
    /// Record this parameter value.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token) {
        if self.repeatable() {
            unimplemented!();
        } else {
            parser.parameters.insert(self.name().clone(), token.text.to_string());
        }
    }

    /// By default a `SimpleParameterNode` completes only to itself.
    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        Completion::new(self.help_symbol().clone(),
                        self.help_text().clone(),
                        token,
                        true,
                        vec![],
                        vec![])
    }

    /// Simple parameters can match any token by default.
    fn matches(&self, _parser: &Parser, _token: Token) -> bool {
        true
    }
}

impl RepeatableNode for SimpleParameterNode {
    #[doc(hidden)]
    fn repeatable_data(&self) -> &RepeatableNodeFields {
        &self.repeatable_fields
    }
}

impl ParameterNode for SimpleParameterNode {
    #[doc(hidden)]
    fn parameter_data(&self) -> &ParameterNodeFields {
        &self.parameter_fields
    }
}

impl SimpleParameterNode {
    /// Construct a new `SimpleParameterNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               required: bool)
               -> Self {
        SimpleParameterNode {
            node_fields: NodeFields {
                name: name.to_string(),
                help_symbol: name.to_string(),
                help_text: help_text.unwrap_or("Parameter").to_string(),
                hidden: hidden,
                priority: priority,
                successors: successors,
            },
            repeatable_fields: RepeatableNodeFields {
                repeatable: repeatable,
                repeat_marker: repeat_marker,
            },
            parameter_fields: ParameterNodeFields { required: required },
        }
    }
}
