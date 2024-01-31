pub struct ThroughputSampler<'a, T: Copy> {
    pub report_interval: std::time::Duration,
    queue: &'a ma_queues::Queue<T>,
}

impl<'a, T: Copy> From<&'a ma_queues::Queue<T>> for ThroughputSampler<'a, T> {
    fn from(queue: &'a ma_queues::Queue<T>) -> Self {
        Self {
            report_interval: std::time::Duration::from_secs(1),
            queue,
        }
    }
}

impl<'a, T: Copy> ThroughputSampler<'a, T> {
    pub fn run(
        self,
        s: &'a std::thread::Scope<'a, 'static>,
    ) -> std::thread::ScopedJoinHandle<'a, ()> {
        s.spawn(move || loop {
            let curt = ma_time::Instant::now();
            let curpos = self.queue.count();
            std::thread::sleep(self.report_interval);
            let delta = self.queue.count() - curpos;
            log::debug!(
                "SPMPC throughput: {} msg/ms",
                delta as u64 * 1_000_000 / curt.elapsed().0
            );
        })
    }
}

