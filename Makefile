.PHONY: all clean install
all: target/release/ulid

install:
	cargo install --path=.

clean:
	cargo clean

target/release/ulid: src/lib.rs src/main.rs
	cargo build --release