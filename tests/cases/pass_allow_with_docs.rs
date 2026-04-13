use strict_typing::strict_types;

/// # Strictness
///
/// - [bool] simple on/off flag, newtype adds no clarity.
#[strict_types(allow(bool))]
struct Flags {
    enabled: bool,
}

fn main() {}
