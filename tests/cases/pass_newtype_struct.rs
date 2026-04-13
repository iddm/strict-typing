use strict_typing::strict_types;

struct Percentage(u8);
struct Label(String);

#[strict_types]
struct Config {
    ratio: Percentage,
    name: Label,
}

fn main() {}
