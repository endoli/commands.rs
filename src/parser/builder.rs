// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Node Builder
//!
//! ```
//! use commands::parser::builder::*;
//!
//! let mut tree = CommandTree::new();
//! tree.command(Command::new("again")
//!                  .hidden(false)
//!                  .parameter(Parameter::new("test")
//!                                 .required(false)
//!                                 .help("This is just a test parameter.")
//!                                 .finalize())
//!                  .finalize());
//! ```

use std::rc::Rc;
use super::nodes::*;

/// Indicate the type of parameter, so that the correct class and node
/// structures are created.
#[derive(Clone)]
pub enum ParameterKind {
    /// This parameter is a `FlagParameter`.
    Flag,
    /// This parameter is a `NamedParameter`.
    Named,
    /// This parameter is a `SimpleParameter`.
    Simple,
}

/// Store a command tree while populating it. This can be used
/// to construct a `RootNode` to be used with the `Parser`.
pub struct CommandTree {
    commands: Vec<Command>,
}

impl CommandTree {
    /// Create a new `CommandTree`.
    pub fn new() -> Self {
        CommandTree { commands: vec![] }
    }

    /// Add a `Command` to the `CommandTree`.
    pub fn command(&mut self, command: Command) {
        self.commands.push(command);
    }

    fn build_parameter(self, parameter: Parameter) -> Rc<Node> {
        match parameter.parameter_kind {
            ParameterKind::Flag => Rc::new(self.build_flag_parameter(parameter)),
            ParameterKind::Named => Rc::new(self.build_named_parameter(parameter)),
            ParameterKind::Simple => Rc::new(self.build_simple_parameter(parameter)),
        }
    }

    fn build_flag_parameter(self, parameter: Parameter) -> FlagParameterNode {
        FlagParameterNode::new(&*parameter.name,
                               parameter.help_text,
                               parameter.hidden,
                               parameter.priority,
                               vec![],
                               parameter.repeatable,
                               None,
                               parameter.required)
    }

    fn build_named_parameter(self, parameter: Parameter) -> NamedParameterNode {
        NamedParameterNode::new(&*parameter.name,
                                parameter.help_text,
                                parameter.hidden,
                                parameter.priority,
                                vec![],
                                parameter.repeatable,
                                None,
                                parameter.required)
    }

    fn build_simple_parameter(self, parameter: Parameter) -> SimpleParameterNode {
        SimpleParameterNode::new(&*parameter.name,
                                 parameter.help_text,
                                 parameter.hidden,
                                 parameter.priority,
                                 vec![],
                                 parameter.repeatable,
                                 None,
                                 parameter.required)
    }
}

/// Description of a command to be added to the `CommandTree`.
#[derive(Clone)]
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
    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    /// Give the command a priority. This is used when sorting
    /// out conflicts during matching and completion.
    pub fn priority(&mut self, priority: i32) -> &mut Self {
        self.priority = priority;
        self
    }

    /// Supply help text for the command.
    pub fn help(&mut self, help_text: &str) -> &mut Self {
        self.help_text = Some(help_text.to_string());
        self
    }

    /// Add a `Parameter` to the command.
    pub fn parameter(&mut self, parameter: Parameter) -> &mut Self {
        self.parameters.push(parameter);
        self
    }

    /// Create a `WrapperNode` instead of a `CommandNode`. The
    /// `wrapped_root` signifies the path to the command that should
    /// be wrapped by this command.
    pub fn wraps(&mut self, wrapped_root: String) -> &mut Self {
        self.wrapped_root = Some(wrapped_root);
        self
    }

    /// Return an instance of `Command` that can be passed to the
    /// `CommandTree`. This is used to terminate the series of construction
    /// methods used to initialize and configure the command.
    pub fn finalize(&self) -> Self {
        self.clone()
    }
}

/// Description of a parameter to be added to the `Command`.
#[derive(Clone)]
pub struct Parameter {
    hidden: bool,
    priority: i32,
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
            priority: PRIORITY_PARAMETER,
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
    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    /// Give the parameter a priority. This is used when sorting
    /// out conflicts during matching and completion.
    pub fn priority(&mut self, priority: i32) -> &mut Self {
        self.priority = priority;
        self
    }

    /// Establish whether or not this parameter is repeatable.
    /// Repeated parameters produce a vector of values and can
    /// be given multiple times within a single command invocation.
    pub fn repeatable(&mut self, repeatable: bool) -> &mut Self {
        self.repeatable = repeatable;
        self
    }

    /// Add an alias that this parameter can use.
    pub fn alias(&mut self, alias: &str) -> &mut Self {
        self.aliases.push(alias.to_string());
        self
    }

    /// Supply the help text for the parameter.
    pub fn help(&mut self, help_text: &str) -> &mut Self {
        self.help_text = Some(help_text.to_string());
        self
    }

    /// Establish whether or not this parameter is required.
    pub fn required(&mut self, required: bool) -> &mut Self {
        self.required = required;
        self
    }

    /// Set which type of `ParameterNode` is supposed to be created
    /// to represent this parameter.
    pub fn kind(&mut self, kind: ParameterKind) -> &mut Self {
        self.parameter_kind = kind;
        self
    }

    /// Return an instance of `Parameter` that can be passed to the
    /// `Command`. This is used to terminate the series of construction
    /// methods used to initialize and configure the parameter.
    pub fn finalize(&self) -> Self {
        self.clone()
    }
}
