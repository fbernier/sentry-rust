//! Transaction & Span Benchmarks
//!
//! Run with: `cargo bench -p sentry-core -- transaction`

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use sentry::protocol::Request;
#[cfg(not(feature = "client"))]
use sentry_core as sentry;

/// Returns a Hub with a no-op transport and the given trace sample rate.
#[cfg(feature = "client")]
fn hub_with_sample_rate(traces_sample_rate: f32) -> sentry::Hub {
    struct NoopTransport;

    impl sentry::Transport for NoopTransport {
        fn send_envelope(&self, envelope: sentry::Envelope) {
            drop(envelope);
        }
    }

    let client = Arc::new(sentry::Client::from(sentry::ClientOptions {
        dsn: Some("https://public@sentry.invalid/1".parse().unwrap()),
        transport: Some(Arc::new(Arc::new(NoopTransport))),
        traces_sample_rate,
        ..Default::default()
    }));
    let scope = Arc::new(sentry::Scope::default());
    sentry::Hub::new(Some(client), scope)
}

fn transaction_lifecycle_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction-lifecycle");

    #[cfg(feature = "client")]
    {
        let hub_sampled = Arc::new(hub_with_sample_rate(1.0));
        let hub_unsampled = Arc::new(hub_with_sample_rate(0.0));

        group.bench_function("start-finish-sampled", |b| {
            sentry::Hub::run(hub_sampled.clone(), || {
                b.iter(|| {
                    let tx = sentry::start_transaction(sentry::TransactionContext::new(
                        "bench", "bench.op",
                    ));
                    tx.finish();
                })
            })
        });

        group.bench_function("start-finish-unsampled", |b| {
            sentry::Hub::run(hub_unsampled.clone(), || {
                b.iter(|| {
                    let tx = sentry::start_transaction(sentry::TransactionContext::new(
                        "bench", "bench.op",
                    ));
                    tx.finish();
                })
            })
        });

        group.bench_function("start-child-span", |b| {
            sentry::Hub::run(hub_sampled.clone(), || {
                let tx =
                    sentry::start_transaction(sentry::TransactionContext::new("bench", "bench.op"));
                b.iter(|| {
                    let span = tx.start_child("child.op", "child span");
                    span.finish();
                });
                tx.finish();
            })
        });

        group.bench_function("transaction-with-10-spans", |b| {
            sentry::Hub::run(hub_sampled.clone(), || {
                b.iter(|| {
                    let tx = sentry::start_transaction(sentry::TransactionContext::new(
                        "bench", "bench.op",
                    ));
                    for i in 0..10 {
                        let span = tx.start_child("child.op", &format!("span {i}"));
                        span.finish();
                    }
                    tx.finish();
                })
            })
        });
    }

    group.finish();
}

fn transaction_metadata_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction-metadata");

    #[cfg(feature = "client")]
    {
        let hub = Arc::new(hub_with_sample_rate(1.0));

        group.bench_function("set-request-and-origin", |b| {
            sentry::Hub::run(hub.clone(), || {
                let tx =
                    sentry::start_transaction(sentry::TransactionContext::new("bench", "bench.op"));
                b.iter(|| {
                    let request = Request {
                        method: Some("GET".into()),
                        url: Some("https://example.com/api/test".parse().unwrap()),
                        ..Default::default()
                    };
                    tx.set_request_and_origin(request, "auto.http.bench");
                });
                tx.finish();
            })
        });

        group.bench_function("set-request-separate", |b| {
            sentry::Hub::run(hub.clone(), || {
                let tx =
                    sentry::start_transaction(sentry::TransactionContext::new("bench", "bench.op"));
                b.iter(|| {
                    let request = Request {
                        method: Some("GET".into()),
                        url: Some("https://example.com/api/test".parse().unwrap()),
                        ..Default::default()
                    };
                    tx.set_request(request);
                    tx.set_origin("auto.http.bench");
                });
                tx.finish();
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    transaction_lifecycle_benchmark,
    transaction_metadata_benchmark
);
criterion_main!(benches);
