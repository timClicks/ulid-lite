.PHONY: all clean test install
all: target/release/libulid.rlib
all: target/release/libulid.so
all: target/release/ulid
all: lib/ulid.h
#all: lib/ulid.hpp

install:
	cargo install --path=.

test: lib/ulid.h
test: target/release/libulid.rlib
test: target/release/libulid.so
test: target/release/ulid
	# tests are serialized until #8 is resolved
	cargo test -- --test-threads=1
	# isolation is disabled to access the system clock
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test

clean:
	cargo clean

target/release/ulid: src/lib.rs src/main.rs
	cargo build --release --bin ulid

target/release/libulid.rlib: src/lib.rs
	cargo build --features=ffi --release --lib

target/release/libulid.so: src/lib.rs
	cargo build --features=ffi --release --lib

lib/ulid.h: src/lib.rs cbindgen.toml
	cbindgen --config=cbindgen.toml > lib/ulid.h

#lib/ulid.hpp: src/lib.rs
#	cbindgen -l c++ > lib/ulid.hpp
