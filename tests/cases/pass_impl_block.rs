use strict_typing::strict_types;

struct Id(u64);
struct Score(f64);

struct Engine;

#[strict_types]
impl Engine {
    fn score(&self, id: Id) -> Score {
        let _ = id;
        Score(42.0)
    }
}

fn main() {}
