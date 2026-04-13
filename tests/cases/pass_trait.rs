use strict_typing::strict_types;

struct Id(u64);
struct Score(f64);

#[strict_types]
trait Scorer {
    fn score(&self, id: Id) -> Score;
}

fn main() {}
