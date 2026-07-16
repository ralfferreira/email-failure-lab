use std::{hint::black_box, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use email_failure_core::{explain, parse_failure, InputSource, ParseInput};
use serde::Deserialize;

const SHORT_SMTP: &str = "550 5.1.1 User unknown";
const MULTILINE_BOUNCE: &str = include_str!("../fixtures/raw/plain-bounce.eml");
const FIXTURE_CASES: &str = include_str!("../fixtures/cases.json");

#[derive(Deserialize)]
struct FixtureCase {
    input: String,
}

fn fixture_inputs() -> Vec<String> {
    serde_json::from_str::<Vec<FixtureCase>>(FIXTURE_CASES)
        .expect("fixture corpus should be valid JSON")
        .into_iter()
        .map(|fixture| fixture.input)
        .collect()
}

fn parse_benchmarks(criterion: &mut Criterion) {
    let fixture_inputs = fixture_inputs();
    let mut group = criterion.benchmark_group("parse");

    group.bench_function("short_smtp", |bencher| {
        bencher.iter(|| {
            black_box(parse_failure(ParseInput {
                raw: black_box(SHORT_SMTP),
                source: InputSource::Inline,
            }))
        });
    });

    group.bench_function("multiline_bounce", |bencher| {
        bencher.iter(|| {
            black_box(parse_failure(ParseInput {
                raw: black_box(MULTILINE_BOUNCE),
                source: InputSource::Inline,
            }))
        });
    });

    group.throughput(Throughput::Elements(fixture_inputs.len() as u64));
    group.bench_function("fixture_corpus", |bencher| {
        bencher.iter(|| {
            for input in black_box(&fixture_inputs) {
                black_box(parse_failure(ParseInput {
                    raw: black_box(input.as_str()),
                    source: InputSource::Inline,
                }));
            }
        });
    });

    group.finish();
}

fn explain_benchmarks(criterion: &mut Criterion) {
    let fixture_inputs = fixture_inputs();
    let mut group = criterion.benchmark_group("explain");

    group.bench_function("short_smtp", |bencher| {
        bencher.iter(|| {
            black_box(explain(ParseInput {
                raw: black_box(SHORT_SMTP),
                source: InputSource::Inline,
            }))
        });
    });

    group.bench_function("multiline_bounce", |bencher| {
        bencher.iter(|| {
            black_box(explain(ParseInput {
                raw: black_box(MULTILINE_BOUNCE),
                source: InputSource::Inline,
            }))
        });
    });

    group.throughput(Throughput::Elements(fixture_inputs.len() as u64));
    group.bench_function("fixture_corpus", |bencher| {
        bencher.iter(|| {
            for input in black_box(&fixture_inputs) {
                black_box(explain(ParseInput {
                    raw: black_box(input.as_str()),
                    source: InputSource::Inline,
                }));
            }
        });
    });

    group.finish();
}

fn benchmark_config() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(50)
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets = parse_benchmarks, explain_benchmarks
}
criterion_main!(benches);
