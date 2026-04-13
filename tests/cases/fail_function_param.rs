use strict_typing::strict_types;

struct Output(u64);

#[strict_types]
fn bad_param(x: u32) -> Output {
    Output(x as u64)
}

fn main() {}
