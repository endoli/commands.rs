// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Commands
//!
//! This crate provides a command system for use in Rust.
//!
//! It provides a general command system which can be used
//! in a variety of environments, including GUI applications
//! and command line utilities. This is inspired by elements
//! of the Lisp Machine, the Common Lisp Interface Manager
//! (CLIM), router command line interfaces, and the TOPS-20
//! command line among other things.
//!
//! * Commands can be defined and grouped into command tables.
//! * Commands can be hooked up with a [`Parser`] for implementing
//!   command line interfaces with completion and parameter validation.
//!
//! This library is in the early stages of development and
//! not everything works yet.
//!
//! [`Parser`]: parser/struct.Parser.html

#![warn(missing_docs)]
#![deny(trivial_numeric_casts,
        unsafe_code, unstable_features,
        unused_import_braces, unused_qualifications)]

pub mod command_table;
pub mod menu_definition;
pub mod parser;
pub mod tokenizer;
pub mod util;
