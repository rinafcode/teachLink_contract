use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_escrow_creation(c: &mut Criterion) {
    c.bench_function("escrow_creation", |b| {
        b.iter(|| {
            let amount = black_box(1000i128);
            let signers = black_box(3u32);
            let threshold = black_box(2u32);
            amount + signers as i128 + threshold as i128
        });
    });
}

fn benchmark_escrow_approval(c: &mut Criterion) {
    c.bench_function("escrow_approval", |b| {
        b.iter(|| {
            let escrow_id = black_box(1u64);
            let signer_count = black_box(3u32);
            escrow_id + signer_count as u64
        });
    });
}

criterion_group!(benches, benchmark_escrow_creation, benchmark_escrow_approval);
criterion_main!(benches);
