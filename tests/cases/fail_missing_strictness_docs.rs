use strict_typing::strict_types;

#[strict_types(allow(bool))]
struct Bad {
    enabled: bool,
}

fn main() {}
