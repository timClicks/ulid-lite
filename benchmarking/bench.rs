use bencher::{benchmark_group, benchmark_main, Bencher};
use ulid;

fn benchmark_serialized(b: &mut Bencher) {
    let mut gen = ulid::UlidGenerator::new();
    b.iter(move || gen.ulid().to_string() )
}

fn benchmark_generation(b: &mut Bencher) {
    let mut gen = ulid::UlidGenerator::new();
    b.iter(move || gen.ulid() )
}

benchmark_group!(
    ulid_lite_perf,
    benchmark_serialized,
    benchmark_generation
);

benchmark_main!(ulid_lite_perf);