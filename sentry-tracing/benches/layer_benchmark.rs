//! Sentry Tracing Layer Benchmarks
//!
//! Run with: `cargo bench -p sentry-tracing`

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Returns a Hub with a no-op transport and 100% trace sampling.
fn tracing_hub() -> sentry::Hub {
    struct NoopTransport;

    impl sentry::Transport for NoopTransport {
        fn send_envelope(&self, envelope: sentry::Envelope) {
            drop(envelope);
        }
    }

    let client = Arc::new(sentry::Client::from(sentry::ClientOptions {
        dsn: Some("https://public@sentry.invalid/1".parse().unwrap()),
        transport: Some(Arc::new(Arc::new(NoopTransport))),
        traces_sample_rate: 1.0,
        ..Default::default()
    }));
    let scope = Arc::new(sentry::Scope::default());
    sentry::Hub::new(Some(client), scope)
}

/// Sets up a tracing subscriber with SentryLayer and runs the closure.
fn with_sentry_subscriber(f: impl FnOnce()) {
    let subscriber = Registry::default().with(sentry_tracing::layer());
    tracing::subscriber::with_default(subscriber, f);
}

fn tracing_events_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracing-events");

    group.bench_function("event-no-client", |b| {
        with_sentry_subscriber(|| {
            b.iter(|| tracing::error!("benchmark error event"))
        });
    });

    let hub = Arc::new(tracing_hub());

    group.bench_function("event-with-client", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                b.iter(|| tracing::error!("benchmark error event"))
            });
        })
    });

    group.bench_function("breadcrumb-event", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                b.iter(|| tracing::info!("benchmark breadcrumb"))
            });
        })
    });

    group.bench_function("mixed-events-batch", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                b.iter(|| {
                    tracing::info!("breadcrumb 1");
                    tracing::info!("breadcrumb 2");
                    tracing::info!("breadcrumb 3");
                    tracing::error!("error event");
                })
            });
        })
    });

    group.finish();
}

fn tracing_spans_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracing-spans");

    group.bench_function("span-lifecycle-no-client", |b| {
        with_sentry_subscriber(|| {
            b.iter(|| {
                let span = tracing::info_span!("bench-span");
                let _enter = span.enter();
            })
        });
    });

    let hub = Arc::new(tracing_hub());

    group.bench_function("span-lifecycle-with-client", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                b.iter(|| {
                    let span = tracing::info_span!("bench-span");
                    let _enter = span.enter();
                })
            });
        })
    });

    group.bench_function("span-enter-exit-repeated", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                let span = tracing::info_span!("poll-span");
                b.iter(|| {
                    for _ in 0..20 {
                        let _enter = span.enter();
                    }
                })
            });
        })
    });

    group.bench_function("nested-spans", |b| {
        sentry::Hub::run(hub.clone(), || {
            with_sentry_subscriber(|| {
                b.iter(|| {
                    let s1 = tracing::info_span!("level-1");
                    let _e1 = s1.enter();
                    let s2 = tracing::info_span!("level-2");
                    let _e2 = s2.enter();
                    let s3 = tracing::info_span!("level-3");
                    let _e3 = s3.enter();
                    let s4 = tracing::info_span!("level-4");
                    let _e4 = s4.enter();
                    let s5 = tracing::info_span!("level-5");
                    let _e5 = s5.enter();
                })
            });
        })
    });

    group.finish();
}

criterion_group!(benches, tracing_events_benchmark, tracing_spans_benchmark);
criterion_main!(benches);
