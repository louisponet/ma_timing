use std::fmt::Display;

pub mod messages;
pub mod throughput;
#[cfg(feature = "timekeeper")]
pub mod timekeeper;
#[cfg(feature = "timekeeper")]
pub use timekeeper::TimeKeeper;
pub mod ffi;
pub mod utils;

use ma_time::Instant;
pub use throughput::ThroughputSampler;
/// Where are the latency ma_queues stored
#[cfg(target_os = "windows")]
const QUEUE_DIR: &str = "Global";
#[cfg(target_os = "linux")]
const QUEUE_DIR: &str = "/dev/shm";
/// The size of the latency ma_queues
const QUEUE_SIZE: usize = 2usize.pow(17);

#[repr(C)]
pub struct Timer {
    pub curmsg: messages::TimingMessage,
    timing_producer: ma_queues::Producer<'static, messages::TimingMessage>,
    latency_producer: ma_queues::Producer<'static, messages::TimingMessage>,
}

impl Timer {
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

unsafe impl Send for Timer {}
unsafe impl Sync for Timer {}

impl Timer {
    pub fn start(&mut self) {
        self.set_start(Instant::now());
        #[cfg(target_arch = "x86_64")]
        unsafe{ std::arch::x86_64::_mm_lfence() };
    }
    pub fn start_t(&self) -> &Instant {
        &self.curmsg.start_t
    }
    pub fn stop_t(&self) -> &Instant {
        &self.curmsg.stop_t
    }
    pub fn stop(&mut self) {
        self.set_stop(Instant::now());
        self.send_business();
    }
    pub fn stop_and_latency(&mut self, ingestion_t: Instant) {
        self.stop();
        self.set_start(ingestion_t);
        self.send_latency();
    }
    pub fn set_stop(&mut self, stop: Instant) {
        self.curmsg.stop_t = stop;
    }
    pub fn set_start(&mut self, start: Instant) {
        self.curmsg.start_t = start;
    }

    pub fn latency(&mut self, ingestion_t: Instant) {
        self.set_stop(Instant::now());
        self.set_start(ingestion_t);
        self.send_latency();
    }
    pub fn send_latency(&mut self) {
        self.latency_producer
            .produce(&self.curmsg);
    }

    pub fn send_business(&mut self) {
        self.timing_producer
            .produce(&self.curmsg);
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
