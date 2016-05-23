// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Indicate the type of parameter, so that the correct class and node
/// structures are created.
#[derive(Clone, Copy, PartialEq)]
pub enum ParameterKind {
    /// This parameter is a flag parameter.
    Flag,
    /// This parameter is a named parameter.
    Named,
    /// This parameter is a simple parameter.
    Simple,
}

/// Minimum priority.
pub const PRIORITY_MINIMUM: i32 = -10000;
/// The default priority for a parameter.
pub const PRIORITY_PARAMETER: i32 = -10;
/// The default priority.
pub const PRIORITY_DEFAULT: i32 = 0;
