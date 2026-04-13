use strict_typing::strict_types;

struct Name(String);

/// # Strictness
///
/// - [String] bare strings are too loose for domain names.
#[strict_types(disallow(String))]
struct Person {
    name: Name,
}

fn main() {}
