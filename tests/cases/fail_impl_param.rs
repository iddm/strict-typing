use strict_typing::strict_types;

struct Score(f64);
struct Engine;

#[strict_types]
impl Engine {
    fn score(&self, id: u64) -> Score {
        Score(id as f64)
    }
}

fn main() {}
