use criterion::{criterion_group, criterion_main, Criterion};
use occara::internal::{make_bottle_cpp, make_bottle_rust};
use std::hint::black_box;

fn occara_benchmark(c: &mut Criterion) {
    const WIDTH: f64 = 50.0;
    const HEIGHT: f64 = 70.0;
    const THICKNESS: f64 = 30.0;

    {
        let mut group = c.benchmark_group("Build Bottle");
        group.bench_function("Rust", |b| {
            b.iter(|| black_box(make_bottle_rust(WIDTH, HEIGHT, THICKNESS)))
        });
        group.bench_function("C++", |b| {
            b.iter(|| black_box(make_bottle_cpp(WIDTH, HEIGHT, THICKNESS)))
        });
    }
}

criterion_group!(benches, occara_benchmark);
criterion_main!(benches);
