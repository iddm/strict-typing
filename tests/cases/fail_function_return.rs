use strict_typing::strict_types;

struct Input(u32);

#[strict_types]
fn bad_return(x: Input) -> u64 {
    x.0 as u64
}

fn main() {}
