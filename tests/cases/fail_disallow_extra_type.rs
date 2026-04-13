use strict_typing::strict_types;

/// # Strictness
///
/// - [String] bare strings disallowed.
#[strict_types(disallow(String))]
struct Bad {
    name: String,
}

fn main() {}
