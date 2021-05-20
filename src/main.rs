use ulid::{init, ulid};

fn main() {
    init();

    println!("{}", ulid());
}
