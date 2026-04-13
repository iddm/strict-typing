# strict-typing

A Rust procedural macro that enforces strict typing on struct fields, enum variants, function signatures, trait definitions, and impl blocks. It prevents the use of bare primitive types, encouraging newtype wrappers for clarity and safety.

Rust is all about correctness in my humble opinion. And I want it to stay that way
in my personal experience. I think this is how Rust should have been made. This
should be a compiler built-in.

## Motivation

Bare primitives like `u32` or `bool` compile fine but carry no domain meaning. A `u32` could be a user ID, a pixel count, or a port number — the type system won't stop you from mixing them up. Newtype wrappers (`struct UserId(u32)`) make intent explicit and catch misuse at compile time.

In my projects, I always create the transparent newtypes that **precisely** define what they carry.
If I ever need to, I also create only the correct versions of conversions between such and other types.

Originally wrote for my own projects, I decided to share it with others. Enjoy.

`#[strict_types]` turns this from a convention into a compiler-enforced rule.

P.S. I decided to name the project `strict-typing` initially with the intention to
extend its functionality.

## Usage

Add the dependency:

```toml
[dependencies]
strict-typing = "0.1"
```

### Basic — reject primitives in struct fields

```rust
use strict_typing::strict_types;

struct Percentage(u8);
struct Label(String);

#[strict_types]
struct Config {
    ratio: Percentage, // OK — newtype wrapper
    name: Label,       // OK
    // count: u32,     // compile error: disallowed type `u32`
}
```

### Enums

```rust
use strict_typing::strict_types;

struct Velocity(f64);

#[strict_types]
enum Event {
    Click { x: Velocity, y: Velocity },
    Disconnect,
    // Resize(u32, u32), // compile error
}
```

### Functions, traits, and impl blocks

```rust
use strict_typing::strict_types;

struct Distance(f64);
struct Speed(f64);
struct Duration(f64);

#[strict_types]
fn compute_speed(d: Distance, t: Duration) -> Speed {
    Speed(d.0 / t.0)
}
```

### Extending the disallow list

By default, all numeric primitives plus `bool` and `char` are disallowed. Add more with `disallow(...)`:

```rust
use strict_typing::strict_types;

/// # Strictness
///
/// - [String] disallowed because domain names should use a newtype.
#[strict_types(disallow(String))]
struct Name {
    // value: String, // compile error
}
```

### Allowing specific primitives

Remove types from the default disallow list with `allow(...)`:

```rust
use strict_typing::strict_types;

/// # Strictness
///
/// - [bool] allowed here because a simple flag is sufficient.
#[strict_types(allow(bool))]
struct Flags {
    enabled: bool, // OK
}
```

### Documentation requirement

When using `allow(...)` or `disallow(...)`, you must document each override in a `/// # Strictness` doc section. This forces authors to justify every exception, keeping the decision visible in code review.

## Default disallowed types

`u8` `u16` `u32` `u64` `u128` `usize` `i8` `i16` `i32` `i64` `i128` `isize` `f32` `f64` `bool` `char`

## License

MIT
