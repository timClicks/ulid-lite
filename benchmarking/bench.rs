use bencher::{benchmark_group, benchmark_main, Bencher};
use ulid_lite;

fn benchmark_serialized(b: &mut Bencher) {
    b.iter(|| ulid_lite::ulid() )
}

fn benchmark_generation(b: &mut Bencher) {
    b.iter(|| ulid_lite::Ulid::new() )
}

benchmark_group!(
    ulid_lite_perf,
    benchmark_serialized,
    benchmark_generation
);

benchmark_main!(ulid_lite_perf);