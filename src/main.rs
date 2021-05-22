use ulid::{init, ulid};

fn main() {
    let _seed = init();

    println!("{}", ulid());
}
