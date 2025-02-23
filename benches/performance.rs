//! Performance benchmarks for custom vector implementation
//! 
//! This suite tests various performance characteristics:
//! - Growth patterns and allocation strategies
//! - Common operations (push, pop, insert, remove)
//! - Memory usage patterns
//! - Iterator performance
//! - Bulk operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use custom_vector_objones25::Vec;
use std::time::Duration;

/// Benchmark vector growth patterns and allocation strategies
fn bench_growth_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("growth_patterns");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    
    // Test different sizes to understand scaling characteristics
    for size in [100, 1000, 10000, 100000].iter() {
        // Test consecutive push operations without pre-allocation
        group.bench_with_input(BenchmarkId::new("push_consecutive", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::new();
                for i in 0..size {
                    vec.push(black_box(i));
                }
            });
        });

        // Test performance with pre-allocated capacity
        group.bench_with_input(BenchmarkId::new("reserve_then_push", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::with_capacity(size);
                for i in 0..size {
                    vec.push(black_box(i));
                }
            });
        });

        // Test performance of bulk extension
        group.bench_with_input(BenchmarkId::new("extend_from_iter", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::new();
                vec.extend(0..size);
            });
        });
    }
    group.finish();
}

/// Benchmark common vector operations
fn bench_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("operations");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    
    for size in [1000, 10000, 100000].iter() {
        // Setup base vector for operations
        let mut base_vec = Vec::with_capacity(*size);
        for i in 0..*size {
            base_vec.push(i);
        }
        
        // Test insert/remove operations at different positions
        for &position in &["start", "middle", "end"] {
            let index = match position {
                "start" => 0,
                "middle" => size / 2,
                "end" => size - 1,
                _ => unreachable!(),
            };
            
            group.bench_with_input(
                BenchmarkId::new(format!("insert_{}", position), size), 
                size, 
                |b, &size| {
                    let mut vec = base_vec.clone();
                    b.iter(|| {
                        vec.insert(index, black_box(size));
                        black_box(&vec);
                    });
                }
            );

            group.bench_with_input(
                BenchmarkId::new(format!("remove_{}", position), size),
                size,
                |b, &size| {
                    let mut vec = base_vec.clone();
                    b.iter(|| {
                        vec.remove(index);
                        vec.insert(index, size);
                    });
                }
            );
        }

        // Test bulk operations
        group.bench_with_input(BenchmarkId::new("extend", size), size, |b, &size| {
            let items: Vec<usize> = (0..100).collect();
            b.iter_with_setup(
                || base_vec.clone(),
                |mut vec| {
                    vec.extend(items.iter().cloned());
                    black_box(&vec);
                    vec.truncate(size);
                }
            );
        });

        // Test iterator performance
        group.bench_with_input(BenchmarkId::new("iterate", size), size, |b, &_size| {
            let vec = base_vec.clone();
            b.iter(|| {
                for item in vec.iter() {
                    black_box(item);
                }
            });
        });
    }
    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(2));
    
    for &pattern_size in &[1024, 4096, 16384] {
        // Test grow-shrink patterns
        group.bench_with_input(
            BenchmarkId::new("grow_shrink", pattern_size),
            &pattern_size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::new();
                    // Grow phase
                    for i in 0..size {
                        vec.push(black_box(i));
                    }
                    // Shrink phase
                    while vec.len() > size / 2 {
                        black_box(vec.pop());
                    }
                    // Regrow phase
                    for i in 0..size / 2 {
                        vec.push(black_box(i));
                    }
                });
            },
        );

        // Test clear and reuse pattern
        group.bench_with_input(
            BenchmarkId::new("clear_reuse", pattern_size),
            &pattern_size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = Vec::with_capacity(size);
                    // Fill
                    for i in 0..size {
                        vec.push(black_box(i));
                    }
                    // Clear
                    vec.clear();
                    // Refill
                    for i in 0..size {
                        vec.push(black_box(i));
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = bench_growth_patterns, bench_operations, bench_memory_patterns
);
criterion_main!(benches);
