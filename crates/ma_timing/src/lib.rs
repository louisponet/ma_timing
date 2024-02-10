use std::fmt::Display;

pub mod messages;
pub mod throughput;
#[cfg(feature = "timekeeper")]
pub mod timekeeper;
#[cfg(feature = "timekeeper")]
pub use timekeeper::TimeKeeper;
pub mod ffi;
pub mod utils;

use ma_time::{Instant, Nanos};
pub use throughput::ThroughputSampler;
/// Where are the latency ma_queues stored
#[cfg(target_os = "windows")]
const QUEUE_DIR: &'static str = "Global";
#[cfg(target_os = "linux")]
const QUEUE_DIR: &'static str = "/dev/shm";
/// The size of the latency ma_queues
const QUEUE_SIZE: usize = 4096;

#[repr(C)]
pub struct Timer<'a> {
    pub curmsg: messages::TimingMessage,
    timing_producer: ma_queues::Producer<'a, messages::TimingMeasurement>,
    latency_producer: ma_queues::Producer<'a, messages::LatencyMeasurement>,
}

impl<'a> Timer<'a> {
    pub fn new<S: Display>(name: S) -> Self {
        let _ = std::fs::create_dir(QUEUE_DIR);
        let timing_queue = ma_queues::Queue::shared(
            format!("{QUEUE_DIR}/timing-{name}"),
            QUEUE_SIZE,
            ma_queues::QueueType::SPMC,
        )
        .expect("couldn't open timing queue");
        let latency_queue = ma_queues::Queue::shared(
            format!("{QUEUE_DIR}/latency-{name}"),
            QUEUE_SIZE,
            ma_queues::QueueType::SPMC,
        )
        .expect("couldn't open latency queue");

        Timer {
            curmsg: Default::default(),
            timing_producer: ma_queues::Producer::from(timing_queue),
            latency_producer: ma_queues::Producer::from(latency_queue),
        }
    }
}

unsafe impl Send for Timer<'_> {}
unsafe impl Sync for Timer<'_> {}

impl<'a> Timer<'a> {
    #[inline(always)]
    pub fn start(&mut self) {
        self.curmsg.start_t = ma_time::Instant::now();
    }
    #[inline(always)]
    pub fn start_t(&self) -> &Instant {
        &self.curmsg.start_t
    }
    #[inline(always)]
    pub fn stop(&mut self) {
        self.curmsg.stop_t = ma_time::Instant::now();
        self.timing_producer
            .produce(&messages::TimingMeasurement::TwoStamps(self.curmsg));
    }
    #[inline(always)]
    pub fn latency(&mut self, ingestion_t: Instant) {
        let curt = Instant::now();
        self.latency_producer
            .produce(&messages::LatencyMeasurement::TwoStamps(
                messages::LatencyMessage {
                    ingestion_t,
                    arrival_t: curt,
                },
            ));
    }
    #[inline(always)]
    pub fn latency_start(&mut self, ingestion_t: Instant) {
        let curt = Instant::now();
        self.curmsg.start_t = curt;
        self.latency_producer
            .produce(&messages::LatencyMeasurement::TwoStamps(
                messages::LatencyMessage {
                    ingestion_t,
                    arrival_t: curt,
                },
            ));
    }
    // pretty hacky
    #[inline(always)]
    pub fn latency_nanos(&mut self, nanos: Nanos) {
        self.latency_producer
            .produce(&messages::LatencyMeasurement::Interval(nanos));
    }
}

pub fn init_logger() {
    fern::Dispatch::new()
        .format(|out, message, _| out.finish(format_args!("{}", message)))
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
