use criterion::{criterion_group, criterion_main, Criterion};
use std::{thread, time::Duration};

use child_wait_timeout::ChildWT;

fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::new(200, 0)) // Set measurement time to 60 seconds
}

fn name() -> String {
    #[cfg(windows)]
    {
        "windows".to_string()
    }
    #[cfg(all(unix, feature = "pidfd"))]
    {
        "unix_pidfd".to_string()
    }
    #[cfg(all(
        unix,
        any(
            all(feature = "thread", not(feature = "pidfd")),
            all(
                not(feature = "signal"),
                not(feature = "thread"),
                not(feature = "pidfd")
            )
        )
    ))]
    {
        "unix_thread".to_string()
    }
    #[cfg(all(
        unix,
        feature = "signal",
        not(feature = "thread"),
        not(feature = "pidfd")
    ))]
    {
        "unix_signal".to_string()
    }
}

fn benchmark_methods_instant(c: &mut Criterion) {
    c.bench_function(format!("wait_timeout_{}_instant", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut child = utilities::sleep_child("0");
                child.wait_timeout(Duration::from_secs(10)).unwrap();
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });

    c.bench_function(format!("wait_{}_instant", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut child = utilities::sleep_child("0");
                child.wait().unwrap();
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });
}

fn benchmark_methods_short(c: &mut Criterion) {
    c.bench_function(format!("wait_timeout_{}_short", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut child = utilities::sleep_child("1");
                child.wait_timeout(Duration::from_secs(10)).unwrap();
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });

    c.bench_function(format!("wait_{}_short", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut child = utilities::sleep_child("1");
                child.wait().unwrap();
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });
}

fn benchmark_methods_timeout(c: &mut Criterion) {
    c.bench_function(format!("wait_timeout_{}_timeout", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut child = utilities::sleep_child("100000");
                let _ = child.wait_timeout(Duration::from_secs(1));
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });

    c.bench_function(format!("sleep_{}_timeout", name()).as_str(), |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                utilities::sleep_child("100000");
                thread::sleep(Duration::from_secs(1));
            }
            let total_duration = start.elapsed();
            total_duration
        })
    });
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = benchmark_methods_instant, benchmark_methods_timeout, benchmark_methods_short
);
criterion_main!(benches);
