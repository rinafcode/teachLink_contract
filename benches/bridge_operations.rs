use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_bridge_deposit(c: &mut Criterion) {
    c.bench_function("bridge_deposit", |b| {
        b.iter(|| {
            // Simulate bridge deposit operation
            let amount = black_box(1000i128);
            let chain_id = black_box(1u32);
            amount + chain_id as i128
        });
    });
}

fn benchmark_bridge_release(c: &mut Criterion) {
    c.bench_function("bridge_release", |b| {
        b.iter(|| {
            // Simulate bridge release operation
            let amount = black_box(1000i128);
            let recipient_count = black_box(1u32);
            amount * recipient_count as i128
        });
    });
}

criterion_group!(benches, benchmark_bridge_deposit, benchmark_bridge_release);
criterion_main!(benches);
