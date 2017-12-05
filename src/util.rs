// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities

use std::cmp;

/// Longest Common Prefix
///
/// Given a vector of string slices, calculate the string
/// slice that is the longest common prefix of the strings.
///
/// ```
/// use commands::util::longest_common_prefix;
///
/// let words = &["zebrawood", "zebrafish", "zebra mussel"];
/// let prefix = longest_common_prefix(words);
/// assert_eq!(prefix, "zebra");
/// ```
pub fn longest_common_prefix<'s>(strings: &'s [&str]) -> &'s str {
    if strings.is_empty() {
        return "";
    }
    let str0 = strings[0];
    let str0bytes = str0.as_bytes();
    let mut len = str0.len();
    for str in &strings[1..] {
        len = cmp::min(
            len,
            str.as_bytes()
                .iter()
                .zip(str0bytes)
                .take_while(|&(a, b)| a == b)
                .count(),
        );
    }
    &strings[0][..len]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_lcp() {
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn single_lcp() {
        assert_eq!(longest_common_prefix(&["ab"]), "ab");
    }

    #[test]
    fn no_lcp() {
        assert_eq!(longest_common_prefix(&["a", "b", "c"]), "");
    }

    #[test]
    fn valid_lcp() {
        // assert_eq!(longest_common_prefix(&["aa", "ab", "ac"]), "a");
        assert_eq!(longest_common_prefix(&["aba", "abb", "abc"]), "ab");
    }

    #[test]
    fn valid_is_shortest_lcp() {
        assert_eq!(longest_common_prefix(&["aba", "ab", "abc"]), "ab");
    }
}
