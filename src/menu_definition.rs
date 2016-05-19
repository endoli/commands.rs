// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Menu Definitions

/// Types of menu items.
pub enum MenuItem {
    /// A separator item in a menu.
    Separator,
}

/// The definition to be used to generate a menu or
/// toolbar in whatever GUI framework is being used.
pub struct MenuDefinition {
    /// The items in the menu. Ordered vector.
    pub items: Vec<MenuItem>,
}
