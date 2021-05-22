.PHONY: all clean install
all: target/release/libulid.rlib
all: target/release/libulid.so
all: target/release/ulid
all: lib/ulid.h
#all: lib/ulid.hpp

install:
	cargo install --path=.

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
