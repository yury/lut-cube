//! Criterion benchmarks for LUT application
//!
//! Benchmarks the apply_lut_auto function with various buffer sizes to measure
//! the performance of SIMD-optimized LUT application.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lut_cube::apply_lut_auto;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Create a test LUT that does a simple transformation
fn create_test_lut() -> [u8; 256] {
    let mut lut = [0u8; 256];
    for i in 0..256 {
        // Create a gamma-like curve for more realistic LUT
        lut[i] = ((i as f64 / 255.0).powf(1.0 / 2.2) * 255.0) as u8;
    }
    lut
}

/// Benchmark LUT application on different buffer sizes
fn bench_lut_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("lut_application");
    
    // Define test sizes: small, 1 MiB, 4 MiB
    let sizes = vec![
        ("small_1KB", 1024),
        ("medium_64KB", 64 * 1024),
        ("large_1MB", 1024 * 1024),
        ("xlarge_4MB", 4 * 1024 * 1024),
    ];
    
    let lut = create_test_lut();
    
    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        
        // Create buffers once and reuse them
        let mut rng = StdRng::seed_from_u64(42);
        let src: Vec<u8> = (0..size).map(|_| rng.r#gen()).collect();
        let mut dst = vec![0u8; size];
        
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, _| {
            b.iter(|| {
                apply_lut_auto(
                    black_box(&src),
                    black_box(&mut dst),
                    black_box(&lut),
                );
            });
        });
    }
    
    group.finish();
}

/// Benchmark with different LUT patterns
fn bench_lut_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("lut_patterns");
    let size = 1024 * 1024; // 1 MiB
    group.throughput(Throughput::Bytes(size as u64));
    
    let mut rng = StdRng::seed_from_u64(42);
    let src: Vec<u8> = (0..size).map(|_| rng.r#gen()).collect();
    let mut dst = vec![0u8; size];
    
    // Identity LUT
    let mut identity_lut = [0u8; 256];
    for i in 0..256 {
        identity_lut[i] = i as u8;
    }
    
    group.bench_function("identity", |b| {
        b.iter(|| {
            apply_lut_auto(
                black_box(&src),
                black_box(&mut dst),
                black_box(&identity_lut),
            );
        });
    });
    
    // Invert LUT
    let mut invert_lut = [0u8; 256];
    for i in 0..256 {
        invert_lut[i] = (255 - i) as u8;
    }
    
    group.bench_function("invert", |b| {
        b.iter(|| {
            apply_lut_auto(
                black_box(&src),
                black_box(&mut dst),
                black_box(&invert_lut),
            );
        });
    });
    
    // Gamma LUT
    let gamma_lut = create_test_lut();
    
    group.bench_function("gamma", |b| {
        b.iter(|| {
            apply_lut_auto(
                black_box(&src),
                black_box(&mut dst),
                black_box(&gamma_lut),
            );
        });
    });
    
    group.finish();
}

/// Benchmark unaligned access patterns
fn bench_lut_alignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("lut_alignment");
    let size = 1024 * 1024; // 1 MiB
    
    let lut = create_test_lut();
    let mut rng = StdRng::seed_from_u64(42);
    let src_full: Vec<u8> = (0..(size + 16)).map(|_| rng.r#gen()).collect();
    let mut dst_full = vec![0u8; size + 16];
    
    // Aligned start
    group.throughput(Throughput::Bytes(size as u64));
    group.bench_function("aligned", |b| {
        let src = &src_full[0..size];
        let dst = &mut dst_full[0..size];
        b.iter(|| {
            apply_lut_auto(
                black_box(src),
                black_box(dst),
                black_box(&lut),
            );
        });
    });
    
    // Unaligned start (offset by 1)
    group.bench_function("unaligned_1", |b| {
        let src = &src_full[1..size + 1];
        let dst = &mut dst_full[1..size + 1];
        b.iter(|| {
            apply_lut_auto(
                black_box(src),
                black_box(dst),
                black_box(&lut),
            );
        });
    });
    
    // Unaligned start (offset by 7)
    group.bench_function("unaligned_7", |b| {
        let src = &src_full[7..size + 7];
        let dst = &mut dst_full[7..size + 7];
        b.iter(|| {
            apply_lut_auto(
                black_box(src),
                black_box(dst),
                black_box(&lut),
            );
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_lut_application, bench_lut_patterns, bench_lut_alignment);
criterion_main!(benches);
