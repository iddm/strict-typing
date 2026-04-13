use strict_typing::strict_types;

struct Velocity(f64);
struct Tag(String);

#[strict_types]
enum Event {
    Click { x: Velocity, y: Velocity },
    Label(Tag),
    Disconnect,
}

fn main() {}
