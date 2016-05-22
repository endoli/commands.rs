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
/// [`RootNode`]: struct.RootNode.html
/// [`Parser`]: struct.Parser.html
pub struct CommandTree {
    commands: Vec<Command>,
}

impl Default for CommandTree {
    fn default() -> Self {
        CommandTree { commands: vec![] }
    }
}

impl CommandTree {
    /// Create a new `CommandTree`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a `Command` to the `CommandTree`.
    pub fn command(&mut self, command: Command) {
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
            Rc::new(CommandNode::new(&*command.name,
                                     command.help_text.clone(),
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
        let p = FlagParameterNode::new(&*parameter.name,
                                       parameter.help_text.clone(),
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
        let p = Rc::new(NamedParameterNode::new(&*parameter.name,
                                                parameter.help_text.clone(),
                                                parameter.hidden,
                                                parameter.priority.unwrap_or(PRIORITY_PARAMETER),
                                                vec![],
                                                parameter.repeatable,
                                                None,
                                                parameter.required));
        parameters.push(p.clone());
        let n = Rc::new(ParameterNameNode::new(&*parameter.name,
                                               parameter.hidden,
                                               PRIORITY_DEFAULT,
                                               vec![p.clone()],
                                               parameter.repeatable,
                                               Some(p.clone()),
                                               p.clone()));
        successors.push(n);
        for alias in &parameter.aliases {
            let a = Rc::new(ParameterNameNode::new(&*alias,
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
        let p = SimpleParameterNode::new(&*parameter.name,
                                         parameter.help_text.clone(),
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
/// [`CommandTree`]: struct.CommandTree.html
pub struct Command {
    hidden: bool,
    priority: i32,
    name: String,
    help_text: Option<String>,
    parameters: Vec<Parameter>,
    wrapped_root: Option<String>,
}

impl Command {
    /// Construct a default (blank) command with the given `name`.
    pub fn new(name: &str) -> Self {
        Command {
            hidden: false,
            priority: PRIORITY_DEFAULT,
            name: name.to_string(),
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
    pub fn help(mut self, help_text: &str) -> Self {
        self.help_text = Some(help_text.to_string());
        self
    }

    /// Add a [`Parameter`] to the command.
    ///
    /// [`Parameter`]: struct.Parameter.html
    pub fn parameter(mut self, parameter: Parameter) -> Self {
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
/// [`Command`]: struct.Command.html
pub struct Parameter {
    hidden: bool,
    priority: Option<i32>,
    name: String,
    repeatable: bool,
    aliases: Vec<String>,
    help_text: Option<String>,
    required: bool,
    parameter_kind: ParameterKind,
}

impl Parameter {
    /// Construct a default (blank) parameter with the given `name`.
    pub fn new(name: &str) -> Self {
        Parameter {
            hidden: false,
            priority: None,
            name: name.to_string(),
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
    pub fn alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_string());
        self
    }

    /// Supply the help text for the parameter.
    pub fn help(mut self, help_text: &str) -> Self {
        self.help_text = Some(help_text.to_string());
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
