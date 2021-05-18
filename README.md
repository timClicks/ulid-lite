# ulid-lite

## About

An implementation of the [ULID] ("Universally Unique Lexicographically Sortable Identifier")
standard.

A ULID is

- 128-bit compatible with UUID
- 1.21e+24 unique ULIDs per millisecond
- Lexicographically sortable!
- Canonically encoded as a 26 character string, as opposed to the 36 character UUID
- Uses Crockford's base32 for better efficiency and readability (5 bits per character)
- Case insensitive
- No special characters (URL safe)

[ULID]: https://github.com/ulid/spec

## Usage

### From the command line

The bundled application generates a ULID and prints it to stdout:

```console
$ ulid
01F5QNHN4G55VHQHA8XG1N6H9H
```

### From Rust

Here is a minimal application that uses this crate:

```rust
use libc;
use ulid_lite::ulid;

fn main() {
    unsafe {
        let now = libc::time(0 as *mut _);
        let now_u32 = (now & u32::MAX as i64) as u32;
        libc::srand(now_u32);
    }

    println!("{}", ulid());
}

```

To correctly use this crate, you need to seed `libc::rand`.

The primary API is the `ulid()` function, which returns a `String`.

```rust
ulid_lite::ulid() -> String
```

For more control, the `ulid::Ulid` type is also available.

```rust
ulid_lite::Ulid::new() -> ulid::Ulid
```

The `Ulid` struct is a wrapper around a `u128`, with a few extra methods.

```rust
let id = ulid_lite::Ulid::new();

// Ulid structs can be converted to strings..
let _: String = id.to_string();

// They implmement Display, LowerHex and UpperHex
println!("{}", id);
println!("{:x}", id);
println!("{:X}", id);
```

More recent ULIDs are higher than older ones:

```rust
use std::thread::sleep;
use std::time::Duration;

let a = ulid();
sleep(Duration::from_millis(1));
let b = ulid();
assert!(a < b);
```



## Installation

At this early stage, this implementation is only available to people
who can install it from source:

```console
$ cargo install --git https://github.com/timClicks/ulid-lite.git
```




## Roadmap

### C library ("libulid"?)

This implementation is designed to make it easy to add a fast
implemention to your language. Accordingly, it'll expose

### PostgreSQL extension

I would like to use this crate to develop pg_ulid extension.


## Warning: Work in progress

A few important features are not yet implemented.

- parsing pre-existing ULIDs 
- monotonicity within the same millisecond
- overflow checks
- no_std: at the moment, this crate uses the `std::time` module to access the clock in a cross-portable manner. Over time, I would like make syscalls directly.


## Why add another crate?

I wanted to implement a crate with a minimalist feel. It is intended to be easy and fast to build.
ulid-lite has minimal dependencies: its only external dependency is `libc`. 
This keeps build times fast and binary size small.

`ulid` does not take a long time to compile:

```console
$ cargo clean
$ cargo build --release
   Compiling libc v0.2.94
   Compiling ulid v0.1.0 (/.../ulid)
    Finished release [optimized] target(s) in 1.44s
```

## Acknowledgements

I've relied on two other implementations to develop `ulid-lite`:

<table>
<tbody>
  <tr>
    <td><a href="http://dylanh.art/">Dylan Hart</a></td>
    <td><a href="https://github.com/dylanhart/ulid-rs">github.com/dylanhart/ulid-rs</a></td>
  </tr>
  <tr>
    <td><a href="https://github.com/mmacedoeu">Marcos Macedo</a></td>
    <td><a href="https://github.com/mmacedoeu/rulid.rs">github.com/mmacedoeu/rulid.rs</a></td>
  </tr>
</tbody>
</table>
