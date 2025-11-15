use cli_testing_specialist::types::CliAnalysis;
use cli_testing_specialist::utils::io_optimized::{
    read_json_naive, read_json_optimized, write_json_naive, write_json_optimized,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::NamedTempFile;

/// Small test data (~1KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallData {
    name: String,
    value: i32,
    items: Vec<String>,
}

fn create_small_data() -> SmallData {
    SmallData {
        name: "test-cli".to_string(),
        value: 42,
        items: (0..10).map(|i| format!("item-{}", i)).collect(),
    }
}

/// Medium test data (~50KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumData {
    items: Vec<SmallData>,
}

fn create_medium_data() -> MediumData {
    MediumData {
        items: (0..100).map(|i| SmallData {
            name: format!("item-{}", i),
            value: i,
            items: (0..10).map(|j| format!("sub-{}-{}", i, j)).collect(),
        }).collect(),
    }
}

/// Large test data (~500KB JSON - simulates large CLI analysis)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LargeData {
    items: Vec<MediumData>,
}

fn create_large_data() -> LargeData {
    LargeData {
        items: (0..10).map(|_| create_medium_data()).collect(),
    }
}

fn bench_write_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_write");

    // Small data (~1KB)
    let small_data = create_small_data();
    let small_json = serde_json::to_string_pretty(&small_data).unwrap();
    group.throughput(Throughput::Bytes(small_json.len() as u64));

    group.bench_function(BenchmarkId::new("naive", "small_1kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_naive(black_box(&small_data), temp_file.path()).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "small_1kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_optimized(black_box(&small_data), temp_file.path()).unwrap();
        });
    });

    // Medium data (~50KB)
    let medium_data = create_medium_data();
    let medium_json = serde_json::to_string_pretty(&medium_data).unwrap();
    group.throughput(Throughput::Bytes(medium_json.len() as u64));

    group.bench_function(BenchmarkId::new("naive", "medium_50kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_naive(black_box(&medium_data), temp_file.path()).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "medium_50kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_optimized(black_box(&medium_data), temp_file.path()).unwrap();
        });
    });

    // Large data (~500KB)
    let large_data = create_large_data();
    let large_json = serde_json::to_string_pretty(&large_data).unwrap();
    group.throughput(Throughput::Bytes(large_json.len() as u64));

    group.bench_function(BenchmarkId::new("naive", "large_500kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_naive(black_box(&large_data), temp_file.path()).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "large_500kb"), |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_optimized(black_box(&large_data), temp_file.path()).unwrap();
        });
    });

    group.finish();
}

fn bench_read_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_read");

    // Small data (~1KB)
    let small_data = create_small_data();
    let small_temp = NamedTempFile::new().unwrap();
    write_json_optimized(&small_data, small_temp.path()).unwrap();
    let small_size = fs::metadata(small_temp.path()).unwrap().len();
    group.throughput(Throughput::Bytes(small_size));

    group.bench_function(BenchmarkId::new("naive", "small_1kb"), |b| {
        b.iter(|| {
            let _data: SmallData = read_json_naive(black_box(small_temp.path())).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "small_1kb"), |b| {
        b.iter(|| {
            let _data: SmallData = read_json_optimized(black_box(small_temp.path())).unwrap();
        });
    });

    // Medium data (~50KB)
    let medium_data = create_medium_data();
    let medium_temp = NamedTempFile::new().unwrap();
    write_json_optimized(&medium_data, medium_temp.path()).unwrap();
    let medium_size = fs::metadata(medium_temp.path()).unwrap().len();
    group.throughput(Throughput::Bytes(medium_size));

    group.bench_function(BenchmarkId::new("naive", "medium_50kb"), |b| {
        b.iter(|| {
            let _data: MediumData = read_json_naive(black_box(medium_temp.path())).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "medium_50kb"), |b| {
        b.iter(|| {
            let _data: MediumData = read_json_optimized(black_box(medium_temp.path())).unwrap();
        });
    });

    // Large data (~500KB)
    let large_data = create_large_data();
    let large_temp = NamedTempFile::new().unwrap();
    write_json_optimized(&large_data, large_temp.path()).unwrap();
    let large_size = fs::metadata(large_temp.path()).unwrap().len();
    group.throughput(Throughput::Bytes(large_size));

    group.bench_function(BenchmarkId::new("naive", "large_500kb"), |b| {
        b.iter(|| {
            let _data: LargeData = read_json_naive(black_box(large_temp.path())).unwrap();
        });
    });

    group.bench_function(BenchmarkId::new("optimized", "large_500kb"), |b| {
        b.iter(|| {
            let _data: LargeData = read_json_optimized(black_box(large_temp.path())).unwrap();
        });
    });

    group.finish();
}

fn bench_roundtrip_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_roundtrip");

    // Medium data roundtrip (~50KB)
    let medium_data = create_medium_data();
    let medium_json = serde_json::to_string_pretty(&medium_data).unwrap();
    group.throughput(Throughput::Bytes(medium_json.len() as u64 * 2)); // Write + Read

    group.bench_function("naive", |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_naive(black_box(&medium_data), temp_file.path()).unwrap();
            let _data: MediumData = read_json_naive(black_box(temp_file.path())).unwrap();
        });
    });

    group.bench_function("optimized", |b| {
        b.iter(|| {
            let temp_file = NamedTempFile::new().unwrap();
            write_json_optimized(black_box(&medium_data), temp_file.path()).unwrap();
            let _data: MediumData = read_json_optimized(black_box(temp_file.path())).unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_write_json,
    bench_read_json,
    bench_roundtrip_json
);
criterion_main!(benches);
