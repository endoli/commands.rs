// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Parser nodes represent the grammar that is defined
// by the currently permissible set of commands and their
// parameters.

use std::rc::Rc;

use super::{Completion, Parser};
use super::constants::*;
use tokenizer::Token;

/// Enumeration of node types used to have vectors of `Node` and so on.
pub enum Node {
    /// `Node` variant wrapping a `CommandNode`.
    Command(CommandNode),
    /// `Node` variant wrapping a `ParameterNode`.
    Parameter(ParameterNode),
    /// `Node` variant wrapping a `ParameterNameNode`.
    ParameterName(ParameterNameNode),
    /// `Node` variant wrapping a `RootNode`.
    Root(RootNode),
}

/// The operations that every node must implement.
pub trait NodeOps {
    /// Accept this node with the given `token` as data.
    ///
    /// By default, nothing needs to happen for `accept`.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token, node_ref: &Rc<Node>);

    /// Can this node be accepted in the current parser state?
    /// By default, a node can be accepted when it hasn't been seen yet.
    ///
    /// The `node_ref` is provided so that implementations have access to
    /// the `Rc<Node>` value for the node rather than having to rely upon
    /// `self` which won't be a [`Node`], but the underlying [`CommandNode`]
    /// or similar.
    ///
    /// [`Node`]: enum.Node.html
    /// [`CommandNode`]: struct.CommandNode.html
    fn acceptable(&self, parser: &Parser, node_ref: &Rc<Node>) -> bool;

    /// Given a node and an optional token, provide the completion options.
    ///
    /// By default, completion should complete for the name of the given
    /// node.
    ///
    /// This is the expected behavior for [`CommandNode`],
    /// [`ParameterNameNode`], as well as [`ParameterNode`] where the
    /// [`ParameterKind`] is `Flag`.
    ///
    /// [`CommandNode`]: struct.CommandNode.html
    /// [`ParameterKind`]: enum.ParameterKind.html
    /// [`ParameterNameNode`]: struct.ParameterNameNode.html
    /// [`ParameterNode`]: struct.ParameterNode.html
    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text>;

    /// By default, a node matches a `token` when the name of the
    /// node starts with the `token`.
    ///
    /// This is the expected behavior for [`CommandNode`],
    /// [`ParameterNameNode`], as well as [`ParameterNode`] where the
    /// [`ParameterKind`] is `Flag`.
    ///
    /// [`CommandNode`]: struct.CommandNode.html
    /// [`ParameterKind`]: enum.ParameterKind.html
    /// [`ParameterNameNode`]: struct.ParameterNameNode.html
    /// [`ParameterNode`]: struct.ParameterNode.html
    fn matches(&self, parser: &Parser, token: Token) -> bool;
}

/// A parse tree node.
pub struct TreeNode {
    /// The name of this node.
    pub name: String,
    /// The text used to identify this node in help text.
    /// This is typically the node name, either in plain
    /// form or decorated for parameters.
    pub help_symbol: String,
    /// Help text describing this node.
    pub help_text: String,
    /// Hidden nodes are not completed. This doesn't modify matching.
    pub hidden: bool,
    /// Match and complete priority.
    pub priority: i32,
    /// Whether or not this node can be repeated. A repeated
    /// node can be `accept`ed multiple times.
    pub repeatable: bool,
    /// If present, this node will no longer be `acceptable`.
    pub repeat_marker: Option<Rc<Node>>,
    /// Possible successor nodes. Collected while building.
    pub successors: Vec<Rc<Node>>,
}

/// The root of a command tree.
pub struct RootNode {
    /// [`TreeNode`] data.
    ///
    /// [`TreeNode`]: struct.TreeNode.html
    pub node: TreeNode,
}

/// A node representing a command. Constructed via [`Command`] and [`CommandTree`].
///
/// If `wrapped_root` is set then this node wraps another command.
/// This is used for the help command so that it can complete
/// normal commands. The `successors` will be those of the wrapped node.
///
/// [`Command`]: struct.Command.html
/// [`CommandTree`]: struct.CommandTree.html
pub struct CommandNode {
    /// [`TreeNode`] data.
    ///
    /// [`TreeNode`]: struct.TreeNode.html
    pub node: TreeNode,
    /// The handler which is executed once this node has been accepted.
    pub handler: Option<fn(&node: Node) -> ()>,
    /// Parameter nodes for this command
    pub parameters: Vec<Rc<Node>>,
    /// If present, the command wrapped by this node.
    pub wrapped_root: Option<Rc<Node>>,
}

/// A node that represented the name portion of a named
/// parameter.
pub struct ParameterNameNode {
    /// [`TreeNode`] data.
    ///
    /// [`TreeNode`]: struct.TreeNode.html
    pub node: TreeNode,
    /// The `parameter` named by this node.
    pub parameter: Rc<Node>,
}

/// A node representing a parameter for a command.
pub struct ParameterNode {
    /// [`TreeNode`] data.
    ///
    /// [`TreeNode`]: struct.TreeNode.html
    pub node: TreeNode,
    /// A `required` parameter must be supplied for the
    /// command line being parsed to be valid.
    pub required: bool,
    /// What type of `ParameterKind` this is.
    pub kind: ParameterKind,
}

impl PartialEq for Node {
    /// Nodes are equal based on pointer equality.
    fn eq(&self, other: &Self) -> bool {
        self as *const _ == other as *const _
    }
}

/// The node in the tree of commands and parameters used in the
/// parser.
///
/// This trait defines the core operations which a node must
/// support.
impl Node {
    /// Get the [`TreeNode`] data for a given `Node`.
    ///
    /// [`TreeNode`]: struct.TreeNode.html
    pub fn node(&self) -> &TreeNode {
        match *self {
            Node::Command(ref command) => &command.node,
            Node::Parameter(ref parameter) => &parameter.node,
            Node::ParameterName(ref name) => &name.node,
            Node::Root(ref root) => &root.node,
        }
    }

    /// Get or calculate successors of this node.
    pub fn successors(&self) -> &Vec<Rc<Node>> {
        match *self {
            Node::Root(ref root) => &root.node.successors,
            _ => &self.node().successors,
        }
    }
}

impl NodeOps for Node {
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token, node_ref: &Rc<Node>) {
        match *self {
            Node::Command(ref command) => command.accept(parser, token, node_ref),
            Node::Parameter(ref parameter) => parameter.accept(parser, token, node_ref),
            Node::ParameterName(ref name) => name.accept(parser, token, node_ref),
            Node::Root(ref root) => root.accept(parser, token, node_ref),
        }
    }

    fn acceptable(&self, parser: &Parser, node_ref: &Rc<Node>) -> bool {
        match *self {
            Node::Command(ref command) => command.acceptable(parser, node_ref),
            Node::Parameter(ref parameter) => parameter.acceptable(parser, node_ref),
            Node::ParameterName(ref name) => name.acceptable(parser, node_ref),
            Node::Root(ref root) => root.acceptable(parser, node_ref),
        }
    }

    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        match *self {
            Node::Command(ref command) => command.complete(token),
            Node::Parameter(ref parameter) => parameter.complete(token),
            Node::ParameterName(ref name) => name.complete(token),
            Node::Root(ref root) => root.complete(token),
        }
    }

    fn matches(&self, parser: &Parser, token: Token) -> bool {
        match *self {
            Node::Command(ref command) => command.matches(parser, token),
            Node::Parameter(ref parameter) => parameter.matches(parser, token),
            Node::ParameterName(ref name) => name.matches(parser, token),
            Node::Root(ref root) => root.matches(parser, token),
        }
    }
}

impl RootNode {
    /// Create a new `RootNode`
    pub fn new(successors: Vec<Rc<Node>>) -> Self {
        RootNode {
            node: TreeNode {
                name: "__root__".to_string(),
                help_symbol: "".to_string(),
                help_text: "".to_string(),
                hidden: false,
                priority: PRIORITY_DEFAULT,
                repeat_marker: None,
                repeatable: false,
                successors: successors,
            },
        }
    }
}

/// `RootNode` does not want to perform any actual `NodeOps` as these
/// operations should only be invoked by the `Parser` on successor nodes.
impl NodeOps for RootNode {
    fn accept<'text>(&self, _parser: &mut Parser<'text>, _token: Token, _node_ref: &Rc<Node>) {}

    fn acceptable(&self, _parser: &Parser, _node_ref: &Rc<Node>) -> bool {
        false
    }

    /// A `RootNode` can not be completed.
    fn complete<'text>(&self, _token: Option<Token<'text>>) -> Completion<'text> {
        panic!("BUG: Can not complete a root node.");
    }

    /// A `RootNode` can not be matched.
    fn matches(&self, _parser: &Parser, _token: Token) -> bool {
        panic!("BUG: Can not match a root node.");
    }
}

impl CommandNode {
    /// Construct a new `CommandNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               handler: Option<fn(&node: Node) -> ()>,
               parameters: Vec<Rc<Node>>)
               -> Self {
        CommandNode {
            node: TreeNode {
                name: name.to_string(),
                help_symbol: name.to_string(),
                help_text: help_text.unwrap_or("Command").to_string(),
                hidden: hidden,
                priority: priority,
                repeat_marker: None,
                repeatable: false,
                successors: successors,
            },
            handler: handler,
            parameters: parameters,
            wrapped_root: None,
        }
    }
}

impl NodeOps for CommandNode {
    /// Record this command.
    fn accept<'text>(&self, parser: &mut Parser<'text>, _token: Token, node_ref: &Rc<Node>) {
        if self.handler.is_some() {
            parser.commands.push(node_ref.clone())
        }
    }

    fn acceptable(&self, parser: &Parser, node_ref: &Rc<Node>) -> bool {
        !parser.nodes.contains(node_ref)
    }

    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        Completion::new(self.node.help_symbol.clone(),
                        self.node.help_text.clone(),
                        token,
                        true,
                        vec![&self.node.name],
                        vec![])
    }

    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.node.name.starts_with(token.text)
    }
}

impl ParameterNameNode {
    /// Construct a new `ParameterNameNode`.
    pub fn new(name: &str,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               parameter: Rc<Node>)
               -> Self {
        let param_node = &parameter.node();
        let help_text = param_node.help_text.clone();
        let help_symbol = name.to_string() + " " + param_node.help_symbol.as_str();
        ParameterNameNode {
            node: TreeNode {
                name: name.to_string(),
                help_symbol: help_symbol,
                help_text: help_text,
                hidden: hidden,
                priority: priority,
                repeat_marker: repeat_marker,
                repeatable: repeatable,
                successors: successors,
            },
            parameter: parameter.clone(),
        }
    }
}

impl NodeOps for ParameterNameNode {
    /// Record this command.
    fn accept<'text>(&self, _parser: &mut Parser<'text>, _token: Token, _node_ref: &Rc<Node>) {}

    fn acceptable(&self, parser: &Parser, node_ref: &Rc<Node>) -> bool {
        if self.node.repeatable {
            return true;
        }
        !parser.nodes.contains(node_ref) &&
        match self.node.repeat_marker {
            None => true,
            Some(ref n) => !parser.nodes.contains(n),
        }
    }

    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        Completion::new(self.node.help_symbol.clone(),
                        self.node.help_text.clone(),
                        token,
                        true,
                        vec![&self.node.name],
                        vec![])
    }

    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        self.node.name.starts_with(token.text)
    }
}

impl ParameterNode {
    /// Construct a new `ParameterNode`.
    pub fn new(name: &str,
               help_text: Option<&str>,
               hidden: bool,
               priority: i32,
               successors: Vec<Rc<Node>>,
               repeatable: bool,
               repeat_marker: Option<Rc<Node>>,
               kind: ParameterKind,
               required: bool)
               -> Self {
        let help_symbol = if repeatable {
            String::from("<") + name + ">..."
        } else {
            String::from("<") + name + ">"
        };
        let default_help_text = match kind {
            ParameterKind::Flag => "Flag",
            ParameterKind::Named | ParameterKind::Simple => "Parameter",
        };
        let help_text = help_text.unwrap_or(default_help_text).to_string();
        ParameterNode {
            node: TreeNode {
                name: name.to_string(),
                help_symbol: help_symbol,
                help_text: help_text,
                hidden: hidden,
                priority: priority,
                repeat_marker: repeat_marker,
                repeatable: repeatable,
                successors: successors,
            },
            kind: kind,
            required: required,
        }
    }
}

impl NodeOps for ParameterNode {
    /// Record this parameter value.
    fn accept<'text>(&self, parser: &mut Parser<'text>, token: Token, _node_ref: &Rc<Node>) {
        if self.node.repeatable {
            unimplemented!();
        } else {
            parser.parameters.insert(self.node.name.clone(), token.text.to_string());
        }
    }

    fn acceptable(&self, parser: &Parser, node_ref: &Rc<Node>) -> bool {
        if self.node.repeatable {
            return true;
        }
        !parser.nodes.contains(node_ref) &&
        match self.node.repeat_marker {
            None => true,
            Some(ref n) => !parser.nodes.contains(n),
        }
    }

    /// By default named and simple parameters complete only to the token
    /// being input while flag parameters complete to the name of the flag.
    fn complete<'text>(&self, token: Option<Token<'text>>) -> Completion<'text> {
        match self.kind {
            ParameterKind::Named | ParameterKind::Simple => {
                Completion::new(self.node.help_symbol.clone(),
                                self.node.help_text.clone(),
                                token,
                                true,
                                vec![],
                                vec![])
            }
            ParameterKind::Flag => {
                Completion::new(self.node.help_symbol.clone(),
                                self.node.help_text.clone(),
                                token,
                                true,
                                vec![&self.node.name],
                                vec![])
            }
        }
    }

    fn matches(&self, _parser: &Parser, token: Token) -> bool {
        match self.kind {
            ParameterKind::Named | ParameterKind::Simple => true,
            ParameterKind::Flag => self.node.name.starts_with(token.text),
        }
    }
}
