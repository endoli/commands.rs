//! # Commands
//!
//! This crate provides a command system.
//!
//! * Commands can be defined and grouped into command tables.
//! * Commands can be hooked up with a ``Parser`` for implementing
//!   command line interfaces with completion and parameter validation.

#![warn(missing_docs)]
#![deny(trivial_casts, trivial_numeric_casts,
        unsafe_code, unstable_features,
        unused_import_braces, unused_qualifications)]

pub mod parser;
pub mod tokenizer;
