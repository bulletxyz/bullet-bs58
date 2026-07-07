//! Encoding benchmarks.

use bullet_bs58::{encode32, encode32_append};
use criterion::{Criterion, criterion_group, criterion_main};

fn encode_bench(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("encode32-vec");
        group.bench_function("bs58-rs", |b| {
            let mut output = Vec::with_capacity(48);
            b.iter(|| bs58::encode(&[42u8; 32]).onto(&mut output))
        });
        group.bench_function("bullet-bs58", |b| b.iter(|| encode32(&[42u8; 32])));
    }
    {
        let mut group = c.benchmark_group("encode32-string");
        group.bench_function("bs58-rs", |b| {
            let mut output = String::with_capacity(44);
            b.iter(|| {
                output.clear();
                bs58::encode(&[42u8; 32]).onto(&mut output)
            });
        });
        group.bench_function("bullet-bs58", |b| {
            let mut output = String::with_capacity(44);
            b.iter(|| {
                output.clear();
                encode32_append(&[42u8; 32], &mut output)
            });
        });
    }
}

criterion_group!(benches, encode_bench);
criterion_main!(benches);
