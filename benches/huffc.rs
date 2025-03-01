use criterion::{black_box, criterion_group, criterion_main, Criterion};
use huffc::tally_frequency;

pub fn criterion_benchmark(c: &mut Criterion) {
    let bytes = b"abcdefghij";

    let loops = 1000000;
    let mut vec_ = Vec::with_capacity(bytes.len() * loops);
    for _ in 0..loops {
        vec_.extend_from_slice(bytes);
    }

    c.bench_function("unsafe", |b| b.iter(|| tally_frequency(black_box(&vec_))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
