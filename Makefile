.PHONY: all clean install
all: target/release/libulid.rlib
all: target/release/libulid.so
all: target/release/ulid

install:
	cargo install --path=.

clean:
	cargo clean

target/release/ulid: src/lib.rs src/main.rs
	cargo build --release

target/release/libulid.rlib: src/lib.rs
	cargo build --release --lib

target/release/libulid.so: src/lib.rs
	cargo build --release --lib
