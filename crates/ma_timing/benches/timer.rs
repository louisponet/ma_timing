use std::{
    sync::{self, atomic::Ordering, Arc},
    time::Duration,
};

use core_affinity::CoreId;
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};

fn time(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("timer"));
    group.bench_function("start_stop", |b| {
        let mut timer = ma_timing::Timer::new("test");
        b.iter(|| {
            timer.start();
            timer.stop();
        })
    });
    group.finish();
}
criterion_group!(benches, time);
criterion_main!(benches);
