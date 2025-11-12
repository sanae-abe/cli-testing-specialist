use cli_testing_specialist::analyzer::CliParser;
use cli_testing_specialist::generator::TestGenerator;
use cli_testing_specialist::types::{TestCase, TestCategory};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

/// Benchmark analyze() function with different CLI tools
fn bench_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze");
    group.measurement_time(Duration::from_secs(10));

    // Small CLI: curl (~50-100 options)
    let curl_path = Path::new("/usr/bin/curl");
    if curl_path.exists() {
        group.bench_function("curl", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                black_box(parser.analyze(black_box(curl_path)))
            });
        });
    }

    // Medium CLI: npm (~30-50 options with many subcommands)
    let npm_path = Path::new("/Users/sanae.abe/.nvm/versions/node/v25.0.0/bin/npm");
    if npm_path.exists() {
        group.bench_function("npm", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                black_box(parser.analyze(black_box(npm_path)))
            });
        });
    }

    // Large CLI: docker (~100+ options with many subcommands)
    let docker_path = Path::new("/usr/local/bin/docker");
    if docker_path.exists() {
        group.bench_function("docker", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                black_box(parser.analyze(black_box(docker_path)))
            });
        });
    }

    // Very large CLI: kubectl (100+ subcommands, many options)
    let kubectl_path = Path::new("/usr/local/bin/kubectl");
    if kubectl_path.exists() {
        group.bench_function("kubectl", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                black_box(parser.analyze(black_box(kubectl_path)))
            });
        });
    }

    group.finish();
}

/// Benchmark generate() function with different sizes of analysis results
fn bench_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate");
    group.measurement_time(Duration::from_secs(10));

    // Analyze real CLIs first to get realistic data
    let curl_path = Path::new("/usr/bin/curl");
    if curl_path.exists() {
        let parser = CliParser::new();
        if let Ok(curl_analysis) = parser.analyze(curl_path) {
            let option_count = curl_analysis.metadata.total_options;
            group.throughput(Throughput::Elements(option_count as u64));
            group.bench_with_input(
                BenchmarkId::new("curl", option_count),
                &curl_analysis,
                |b, analysis| {
                    b.iter(|| {
                        let generator = TestGenerator::new(
                            black_box(analysis.clone()),
                            black_box(vec![TestCategory::Basic, TestCategory::Help]),
                        );
                        black_box(generator.generate())
                    });
                },
            );
        }
    }

    let npm_path = Path::new("/Users/sanae.abe/.nvm/versions/node/v25.0.0/bin/npm");
    if npm_path.exists() {
        let parser = CliParser::new();
        if let Ok(npm_analysis) = parser.analyze(npm_path) {
            let option_count = npm_analysis.metadata.total_options;
            group.throughput(Throughput::Elements(option_count as u64));
            group.bench_with_input(
                BenchmarkId::new("npm", option_count),
                &npm_analysis,
                |b, analysis| {
                    b.iter(|| {
                        let generator = TestGenerator::new(
                            black_box(analysis.clone()),
                            black_box(vec![TestCategory::Basic, TestCategory::Help]),
                        );
                        black_box(generator.generate())
                    });
                },
            );
        }
    }

    let docker_path = Path::new("/usr/local/bin/docker");
    if docker_path.exists() {
        let parser = CliParser::new();
        if let Ok(docker_analysis) = parser.analyze(docker_path) {
            let option_count = docker_analysis.metadata.total_options;
            group.throughput(Throughput::Elements(option_count as u64));
            group.bench_with_input(
                BenchmarkId::new("docker", option_count),
                &docker_analysis,
                |b, analysis| {
                    b.iter(|| {
                        let generator = TestGenerator::new(
                            black_box(analysis.clone()),
                            black_box(vec![TestCategory::Basic, TestCategory::Help]),
                        );
                        black_box(generator.generate())
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark JSON serialization of analysis results
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Analyze curl to get realistic data
    let curl_path = Path::new("/usr/bin/curl");
    if curl_path.exists() {
        let parser = CliParser::new();
        if let Ok(analysis) = parser.analyze(curl_path) {
            // Benchmark JSON serialization
            group.bench_function("json_serialize", |b| {
                b.iter(|| black_box(serde_json::to_string(black_box(&analysis))));
            });

            // Benchmark JSON deserialization
            let json_str = serde_json::to_string(&analysis).unwrap();
            group.bench_function("json_deserialize", |b| {
                b.iter(|| {
                    black_box(serde_json::from_str::<
                        cli_testing_specialist::types::CliAnalysis,
                    >(black_box(&json_str)))
                });
            });

            // Benchmark pretty JSON serialization
            group.bench_function("json_serialize_pretty", |b| {
                b.iter(|| black_box(serde_json::to_string_pretty(black_box(&analysis))));
            });
        }
    }

    group.finish();
}

/// Benchmark end-to-end workflow (analyze + generate)
fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(15));

    // Curl end-to-end
    let curl_path = Path::new("/usr/bin/curl");
    if curl_path.exists() {
        group.bench_function("curl_workflow", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                let analysis = parser.analyze(black_box(curl_path)).unwrap();
                let generator = TestGenerator::new(
                    black_box(analysis),
                    black_box(vec![TestCategory::Basic, TestCategory::Help]),
                );
                black_box(generator.generate())
            });
        });
    }

    // npm end-to-end
    let npm_path = Path::new("/Users/sanae.abe/.nvm/versions/node/v25.0.0/bin/npm");
    if npm_path.exists() {
        group.bench_function("npm_workflow", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                let analysis = parser.analyze(black_box(npm_path)).unwrap();
                let generator = TestGenerator::new(
                    black_box(analysis),
                    black_box(vec![TestCategory::Basic, TestCategory::Help]),
                );
                black_box(generator.generate())
            });
        });
    }

    // docker end-to-end
    let docker_path = Path::new("/usr/local/bin/docker");
    if docker_path.exists() {
        group.bench_function("docker_workflow", |b| {
            b.iter(|| {
                let parser = CliParser::new();
                let analysis = parser.analyze(black_box(docker_path)).unwrap();
                let generator = TestGenerator::new(
                    black_box(analysis),
                    black_box(vec![TestCategory::Basic, TestCategory::Help]),
                );
                black_box(generator.generate())
            });
        });
    }

    group.finish();
}

/// Benchmark parallel processing capability
fn bench_parallel(c: &mut Criterion) {
    use rayon::prelude::*;

    let mut group = c.benchmark_group("parallel");
    group.measurement_time(Duration::from_secs(10));

    // Create sample test cases
    let test_cases: Vec<TestCase> = (0..100)
        .map(|i| {
            TestCase::new(
                format!("test-{}", i),
                format!("Test case {}", i),
                TestCategory::Basic,
                format!("test command {}", i),
            )
        })
        .collect();

    // Sequential processing
    group.bench_function("sequential", |b| {
        b.iter(|| {
            test_cases
                .iter()
                .map(|tc| tc.name.len())
                .collect::<Vec<_>>()
        });
    });

    // Parallel processing
    group.bench_function("parallel", |b| {
        b.iter(|| {
            test_cases
                .par_iter()
                .map(|tc| tc.name.len())
                .collect::<Vec<_>>()
        });
    });

    // Test if parallel generation is available
    let curl_path = Path::new("/usr/bin/curl");
    if curl_path.exists() {
        let parser = CliParser::new();
        if let Ok(analysis) = parser.analyze(curl_path) {
            group.bench_function("generate_parallel", |b| {
                b.iter(|| {
                    let generator = TestGenerator::new(
                        black_box(analysis.clone()),
                        black_box(vec![TestCategory::Basic, TestCategory::Help]),
                    );
                    black_box(generator.generate_parallel())
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_analyze,
    bench_generate,
    bench_serialization,
    bench_end_to_end,
    bench_parallel
);
criterion_main!(benches);
