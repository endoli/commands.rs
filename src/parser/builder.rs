// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::rc::Rc;
use super::nodes::*;

/// Indicate the type of parameter, so that the correct class and node
/// structures are created.
#[derive(Clone, Copy, PartialEq)]
pub enum ParameterKind {
    /// This parameter is a `FlagParameter`.
    Flag,
    /// This parameter is a `NamedParameter`.
    Named,
    /// This parameter is a `SimpleParameter`.
    Simple,
}

/// Store a command tree while populating it. This is used
/// to construct a [`RootNode`] to be used with the [`Parser`].
///
/// The lifetime parameter `'a` refers to the lifetime
/// of the strings used for [command] and [parameter] names and
/// help text.
///
/// [command]: struct.Command.html
/// [parameter]: struct.Parameter.html
/// [`Parser`]: struct.Parser.html
/// [`RootNode`]: struct.RootNode.html
pub struct CommandTree<'a> {
    commands: Vec<Command<'a>>,
}

impl<'a> Default for CommandTree<'a> {
    fn default() -> Self {
        CommandTree { commands: vec![] }
    }
}

impl<'a> CommandTree<'a> {
    /// Create a new `CommandTree`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a `Command` to the `CommandTree`.
    pub fn command(&mut self, command: Command<'a>) {
        self.commands.push(command);
    }

    /// Construct the `CommandTree` and produce a `RootNode`.
    pub fn finalize(&self) -> Rc<RootNode> {
        let mut successors: Vec<Rc<Node>> = vec![];
        for c in &self.commands {
            successors.push(self.build_command(c));
        }
        RootNode::new(successors)
    }

    fn build_command(&self, command: &Command) -> Rc<Node> {
        let mut parameters: Vec<Rc<ParameterNode>> = vec![];
        let mut successors: Vec<Rc<Node>> = vec![];
        for parameter in &command.parameters {
            match parameter.parameter_kind {
                ParameterKind::Flag => {
                    self.build_flag_parameter(parameter, &mut parameters, &mut successors);
                }
                ParameterKind::Named => {
                    self.build_named_parameter(parameter, &mut parameters, &mut successors);
                }
                ParameterKind::Simple => {
                    self.build_simple_parameter(parameter, &mut parameters, &mut successors);
                }
            };
        }
        if let Some(_) = command.wrapped_root {
            // XXX: This should be a WrapperNode.
            unimplemented!()
        } else {
            Rc::new(CommandNode::new(command.name,
                                     command.help_text,
                                     command.hidden,
                                     command.priority,
                                     successors,
                                     None,
                                     parameters))
        }
    }

    fn build_flag_parameter(&self,
                            parameter: &Parameter,
                            parameters: &mut Vec<Rc<ParameterNode>>,
                            successors: &mut Vec<Rc<Node>>) {
        let p = FlagParameterNode::new(parameter.name,
                                       parameter.help_text,
                                       parameter.hidden,
                                       parameter.priority.unwrap_or(PRIORITY_DEFAULT),
                                       vec![],
                                       parameter.repeatable,
                                       None,
                                       parameter.required);
        let fp = Rc::new(p);
        parameters.push(fp.clone());
        successors.push(fp);
    }

    fn build_named_parameter(&self,
                             parameter: &Parameter,
                             parameters: &mut Vec<Rc<ParameterNode>>,
                             successors: &mut Vec<Rc<Node>>) {
        let p = Rc::new(NamedParameterNode::new(parameter.name,
                                                parameter.help_text,
                                                parameter.hidden,
                                                parameter.priority.unwrap_or(PRIORITY_PARAMETER),
                                                vec![],
                                                parameter.repeatable,
                                                None,
                                                parameter.required));
        parameters.push(p.clone());
        let n = Rc::new(ParameterNameNode::new(parameter.name,
                                               parameter.hidden,
                                               PRIORITY_DEFAULT,
                                               vec![p.clone()],
                                               parameter.repeatable,
                                               Some(p.clone()),
                                               p.clone()));
        successors.push(n);
        for alias in &parameter.aliases {
            let a = Rc::new(ParameterNameNode::new(alias,
                                                   parameter.hidden,
                                                   PRIORITY_DEFAULT,
                                                   vec![p.clone()],
                                                   parameter.repeatable,
                                                   Some(p.clone()),
                                                   p.clone()));
            successors.push(a);
        }
    }

    fn build_simple_parameter(&self,
                              parameter: &Parameter,
                              parameters: &mut Vec<Rc<ParameterNode>>,
                              successors: &mut Vec<Rc<Node>>) {
        let p = SimpleParameterNode::new(parameter.name,
                                         parameter.help_text,
                                         parameter.hidden,
                                         parameter.priority.unwrap_or(PRIORITY_PARAMETER),
                                         vec![],
                                         parameter.repeatable,
                                         None,
                                         parameter.required);
        let sp = Rc::new(p);
        parameters.push(sp.clone());
        successors.push(sp);
    }
}

/// Description of a command to be added to the [`CommandTree`].
///
/// The lifetime parameter `'a` refers to the lifetime
/// of the strings used for command names and help text.
///
/// [`CommandTree`]: struct.CommandTree.html
pub struct Command<'a> {
    hidden: bool,
    priority: i32,
    name: &'a str,
    help_text: Option<&'a str>,
    parameters: Vec<Parameter<'a>>,
    wrapped_root: Option<String>,
}

impl<'a> Command<'a> {
    /// Construct a default (blank) command with the given `name`.
    pub fn new(name: &'a str) -> Self {
        Command {
            hidden: false,
            priority: PRIORITY_DEFAULT,
            name: name,
            help_text: None,
            parameters: vec![],
            wrapped_root: None,
        }
    }

    /// Mark the command as hidden. Hidden commands will match
    /// within the parser, but are not listed during completion.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Give the command a priority. This is used when sorting
    /// out conflicts during matching and completion.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Supply help text for the command.
    pub fn help(mut self, help_text: &'a str) -> Self {
        self.help_text = Some(help_text);
        self
    }

    /// Add a [`Parameter`] to the command.
    ///
    /// [`Parameter`]: struct.Parameter.html
    pub fn parameter(mut self, parameter: Parameter<'a>) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Create a [`WrapperNode`] instead of a [`CommandNode`]. The
    /// `wrapped_root` signifies the path to the command that should
    /// be wrapped by this command.
    ///
    /// [`CommandNode`]: struct.CommandNode.html
    /// [`WrapperNode`]: struct.WrapperNode.html
    pub fn wraps(mut self, wrapped_root: String) -> Self {
        self.wrapped_root = Some(wrapped_root);
        self
    }
}

/// Description of a parameter to be added to the [`Command`].
///
/// The lifetime parameter `'a` refers to the lifetime
/// of the strings used for parameter names, aliases and
/// help text.
///
/// [`Command`]: struct.Command.html
pub struct Parameter<'a> {
    hidden: bool,
    priority: Option<i32>,
    name: &'a str,
    repeatable: bool,
    aliases: Vec<&'a str>,
    help_text: Option<&'a str>,
    required: bool,
    parameter_kind: ParameterKind,
}

impl<'a> Parameter<'a> {
    /// Construct a default (blank) parameter with the given `name`.
    pub fn new(name: &'a str) -> Self {
        Parameter {
            hidden: false,
            priority: None,
            name: name,
            repeatable: false,
            aliases: vec![],
            help_text: None,
            required: false,
            parameter_kind: ParameterKind::Simple,
        }
    }

    /// Mark the parameter as hidden. Hidden parameters will match
    /// within the parser, but are not listed during completion.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Give the parameter a priority. This is used when sorting
    /// out conflicts during matching and completion.
    ///
    /// The `priority` of a `Parameter` defaults to `PRIORITY_PARAMETER`
    /// except for when the `kind` is `ParameterKind::Flag` in which
    /// case, the default will be `PRIORITY_DEFAULT`.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Establish whether or not this parameter is repeatable.
    /// Repeated parameters produce a vector of values and can
    /// be given multiple times within a single command invocation.
    pub fn repeatable(mut self, repeatable: bool) -> Self {
        self.repeatable = repeatable;
        self
    }

    /// Add an alias that this parameter can use.
    ///
    /// Aliases are currently only valid for parameters of `kind`
    /// `ParameterKind::Named`.
    pub fn alias(mut self, alias: &'a str) -> Self {
        self.aliases.push(alias);
        self
    }

    /// Supply the help text for the parameter.
    pub fn help(mut self, help_text: &'a str) -> Self {
        self.help_text = Some(help_text);
        self
    }

    /// Establish whether or not this parameter is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set which type of [`ParameterNode`] is supposed to be created
    /// to represent this parameter.
    ///
    /// [`ParameterNode`]: trait.ParameterNode.html
    pub fn kind(mut self, kind: ParameterKind) -> Self {
        self.parameter_kind = kind;
        self
    }
}
