use strict_typing::strict_types;

struct Id(u64);

#[strict_types]
trait Bad {
    fn score(&self, id: Id) -> f64;
}

fn main() {}
