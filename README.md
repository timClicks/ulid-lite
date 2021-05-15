# ulid

## About

An implementation of the ULID ("Universally Unique Lexicographically Sortable Identifier")
standard.

A ULID is

- 128-bit compatible with UUID
- 1.21e+24 unique ULIDs per millisecond
- Lexicographically sortable!
- Canonically encoded as a 26 character string, as opposed to the 36 character UUID
- Uses Crockford's base32 for better efficiency and readability (5 bits per character)
- Case insensitive
- No special characters (URL safe)

## Installation

```
$ cargo install --git https://github.com/timClicks/ulid
```


## Warning: Work in progress

A few important features are not yet implemented.

- parsing pre-existing ULIDs 
- monotonicity within the same millisecond
- overflow checks
- no_std: at the moment, this crate uses the `std::time` module to access the clock in a cross-portable manner. Over time, I would like make syscalls directly.


## Why add another crate?

I wanted to implement a crate with a minimalist feel.

- API-compliant with the spec: `ulid::ulid()` should be sufficient.
- minimal dependencies: this implementation's only external dependency is `libc`. 
  That is intended to keep build times fast and build size slim.
