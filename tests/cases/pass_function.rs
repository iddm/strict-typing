use strict_typing::strict_types;

struct Distance(f64);
struct Duration(f64);
struct Speed(f64);

#[strict_types]
fn compute_speed(d: Distance, t: Duration) -> Speed {
    Speed(d.0 / t.0)
}

fn main() {
    let _ = compute_speed(Distance(100.0), Duration(10.0));
}
