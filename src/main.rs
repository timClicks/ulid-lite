use ulid::{UlidGenerator};

fn main() {
    println!("{}", UlidGenerator::new().ulid());
}
