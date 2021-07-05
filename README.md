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
use ulid::ulid;

fn main() {
    println!("{}", ulid());
}
```

The primary API is the `ulid()` function, which returns a `String`.
If you would like access to the individual bits, then call `ulid_raw()`. 

```rust
ulid::ulid() -> String
ulid::ulid_raw() -> u128
```

For more control, the `ulid::Ulid` type is also available.

```rust
ulid::Ulid::new() -> ulid::Ulid
```

The `Ulid` struct is a wrapper around a `u128`, with a few extra methods.

```rust
let id = ulid::Ulid::new();

// They implement Display, LowerHex and UpperHex
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

To generate many `ulid::Ulid` values, you're recommended to use `UlidGenerator`.
It provides the ability to seed the internal pseudo-random number generator.

```rust
// use the system's clock as the initial seed...
let mut ulid_gen = ulid::UlidGenerator::new();
let ulids: Vec<_> = ulid_gen.take(1000).collect();
```

```rust
// ...or use a fixed initial seed
let mut ulid_gen = ulid::UlidGenerator::from_seed(12345);
let ulid = ulid_gen.ulid();
```

### From C

A C API is available at `lib/ulid.h`.  Here is a minimal application that generates and prints a ULID:

```c
#include <stdio.h>
#include "ulid.h"

int main(void) {
    char str[27];

    ulid_ctx ctx = ulid_init(0);
    ulid_write_new(&ctx, str, sizeof(str));

    printf("%s\n", str);

    return 0;
}
```

`libulid` also provides access to creating binary (128 bit)
ULIDs and converting those to strings (this example is
intentionally convoluted to showcase error handling):

```c
#include <stdio.h>
#include "ulid.h"

int main(void) {
    ulid_ctx ctx;
    ulid_t id;
    char buf[64], *cur = buf;
    int n, size = sizeof(buf);

    ctx = ulid_init(0);
    ulid_new(&ctx, &id);

    n = snprintf(cur, size, "Your ULID is ");
    if (n >= size)
        return 1;
    cur += n;
    size -= n;

    n = ulid_write(&id, cur, size);
    if (n < 0) /* failed, typically buffer is too small */
        return 1;
    cur += n;
    size -= n;

    n = snprintf(cur, size, ".");
    if (n >= size)
        return 1;

    printf("%s\n", buf);
    return 0;
}
```

#### Building the C interface

To regenerate the `ulid.h` header file, run `make lib/ulid.h`.

To build the `libulid` shared library, run `make target/release/libulid.so`.

## Installation

At this early stage, this implementation is only available to people
who can install it from source:

```console
$ cargo install --git https://github.com/timClicks/ulid-lite.git
```

## Contributing

You are very welcome to contribute to project in any form, however you must abide by the [Rust Code of Conduct].

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct

### Non-code contributions

Your contribution is important! Please [submit an issue] with your suggested change.

[submit an issue]: https://github.com/timClicks/ulid-lite/issues/new

### Code contributions

> Note: these instructions have only been tested on Ubuntu,
> please submit corrections/improvements for other operating systems.

#### Setting up a development environment

To begin, you require the following tools:

- A Rust installation that includes `rustc`, `rustup`, and `cargo`
- `git`
- `make`


From the root of the project, run `setup-devenv` to install dependencies that are managed by `cargo` or `rustup`, such as [MIRI](https://github.com/rust-lang/miri):

```console
$ ./setup-devenv
```

#### Submitting changes

`ulid-lite` follows the standard GitHub workflow for code changes.
Please fork the project, push commits to that fork and submit a pull request (PR).

Before submitting a PR, you should run `make test && make` from the project's root directory, rather than `cargo test`.
This will ensure that the MIRI tests run correctly and that artifacts can all be built.



## Roadmap

### PostgreSQL extension

I would like to use this crate to develop pg_ulid extension.


### More features

- parsing pre-existing ULIDs 
- monotonicity within the same millisecond
- overflow checks

### More platforms

`ulid-lite` is currently only built for Linux. Patches are welcome to support more platforms.


## Why add another crate?

I wanted to implement a crate with a minimalist feel. It is intended to be easy and fast to build.
`ulid-lite` has minimal dependencies. This keeps build times fast and binary size small.

`ulid` does not take a long time to compile:

```console
$ cargo clean
$ cargo build --release
   Compiling libc v0.2.94
   Compiling lazy_static v0.2.11
   Compiling rand v0.4.6
   Compiling rand v0.3.23
   Compiling xorshift v0.1.3
   Compiling ulid-lite v0.4.0 (/.../ulid-lite)
    Finished release [optimized] target(s) in 5.68s
```

Perhaps more importantly however, `ulid-lite` is very fast.
A single CPU core can generate about 35,700 ULIDs per millisecond. 

```console
$ cargo bench
...
running 2 tests
test benchmark_generation ... bench:          28 ns/iter (+/- 2)
test benchmark_serialized ... bench:          71 ns/iter (+/- 12)
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
