//! Hub & Scope Benchmarks
//!
//! Run with: `cargo bench -p sentry-core -- hub`

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(not(feature = "client"))]
use sentry_core as sentry;

/// Returns a Hub with a no-op transport for benchmarking.
#[cfg(feature = "client")]
fn discarding_hub() -> sentry::Hub {
    struct NoopTransport;

    impl sentry::Transport for NoopTransport {
        fn send_envelope(&self, envelope: sentry::Envelope) {
            drop(envelope);
        }
    }

    let client = Arc::new(sentry::Client::from(sentry::ClientOptions {
        dsn: Some("https://public@sentry.invalid/1".parse().unwrap()),
        transport: Some(Arc::new(Arc::new(NoopTransport))),
        ..Default::default()
    }));
    let scope = Arc::new(sentry::Scope::default());
    sentry::Hub::new(Some(client), scope)
}

fn hub_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hub-operations");

    #[cfg(feature = "client")]
    {
        let hub = Arc::new(discarding_hub());

        group.bench_function("hub-with", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| sentry::Hub::with(|hub| hub.client().is_some()))
            })
        });

        group.bench_function("hub-with-active", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| sentry::Hub::with_active(|hub| hub.client().is_some()))
            })
        });

        group.bench_function("hub-client-arc-clone", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| sentry::Hub::with(|hub| hub.client()))
            })
        });

        group.bench_function("hub-new-from-top", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| sentry::Hub::with(|hub| sentry::Hub::new_from_top(hub)))
            })
        });
    }

    group.finish();
}

fn scope_mutation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scope-mutation");

    #[cfg(feature = "client")]
    {
        let hub = Arc::new(discarding_hub());

        group.bench_function("configure-scope", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| {
                    sentry::configure_scope(|scope| {
                        scope.set_tag("bench-key", "bench-value");
                    })
                })
            })
        });

        group.bench_function("push-pop-scope", |b| {
            sentry::Hub::run(hub.clone(), || {
                b.iter(|| {
                    sentry::Hub::with(|hub| {
                        let _guard = hub.push_scope();
                    })
                })
            })
        });
    }

    group.finish();
}

criterion_group!(benches, hub_operations_benchmark, scope_mutation_benchmark);
criterion_main!(benches);
