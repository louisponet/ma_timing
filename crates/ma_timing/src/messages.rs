use ma_time::{Duration, Instant, TimeStamped};


//TODO: Cleanup can be merged I think
#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct LatencyMessage {
    //ingestion is first creation time
    pub ingestion_t: Instant,
    // arrival_t is when the message was seen by consumer
    pub arrival_t: Instant,
}

impl LatencyMessage {
    #[inline(always)]
    pub fn new(ingestion_t: Instant, arrival_t: Instant) -> Self {
        Self {
            ingestion_t,
            arrival_t,
        }
    }
    pub fn latency(&self) -> Duration {
        Duration(self.arrival_t.0 - self.ingestion_t.0)
    }
}

#[derive(Clone, Copy)]
pub enum LatencyMeasurement {
    TwoStamps(LatencyMessage),
    Interval(Duration)
}
impl Default for LatencyMeasurement {
    fn default() -> Self {
        Self::Interval(Duration::default())
    }
}
impl LatencyMeasurement {
    pub fn latency(&self) -> Duration {
        match self {
            Self::TwoStamps(msg) => msg.latency(),
            Self::Interval(n) => *n
        }
    }
}


#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct TimingMessage {
    pub start_t: Instant,
    pub stop_t: Instant,
}

impl TimingMessage {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            start_t: Instant::now(),
            stop_t: Default::default(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Duration(self.stop_t.0 - self.start_t.0)
    }
}

#[derive(Clone, Copy)]
pub enum TimingMeasurement {
    TwoStamps(TimingMessage),
    Interval(Duration)
}

impl TimingMeasurement {
    pub fn elapsed(&self) -> Duration {
        match self {
            Self::TwoStamps(msg) => msg.elapsed(),
            Self::Interval(n) => *n
        }
    }
}

impl Default for TimingMeasurement {
    fn default() -> Self {
        Self::Interval(Duration::default())
    }
}
/// To perform rudimentary verification for a msg that was found at a given position in the queue
pub trait Verifyable {
    fn verify(&self, pos: usize) -> bool;
}

pub trait Data {
    type Data;
    fn data(&self) -> Self::Data;
}

// Group a bunch of traits together
pub trait TestMsg: 'static + Copy + std::fmt::Debug + Data + Default + TimeStamped {}
impl<T> TestMsg for T where T: 'static + Copy + std::fmt::Debug + Data + Default + TimeStamped {}

#[derive(Clone, Copy, Debug)]
pub struct LongMsg {
    i: [usize; 1024],
    t: Instant,
}

const SHORTMSGSIZE: usize = 42;

// #[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ShortMsg {
    pub i: usize,
    pub t: Instant,
    pub crap: [u8; SHORTMSGSIZE],
}
impl ShortMsg {
    pub fn new(i: usize) -> Self {
        Self {
            i,
            t: Instant::now(),
            crap: [0; SHORTMSGSIZE],
        }
    }
}

// Default
impl Default for LongMsg {
    fn default() -> Self {
        Self {
            i: [0; 1024],
            t: Instant::now(),
        }
    }
}

impl Default for ShortMsg {
    fn default() -> Self {
        Self {
            i: Default::default(),
            t: Instant::now(),
            crap: [0; SHORTMSGSIZE],
        }
    }
}

// Data
impl Data for LongMsg {
    type Data = [usize; 1024];
    fn data(&self) -> Self::Data {
        self.i
    }
}

impl Data for ShortMsg {
    type Data = usize;
    fn data(&self) -> Self::Data {
        self.i
    }
}

// Verifyable
impl Verifyable for LongMsg {
    // For long message we verify that all integers are identical.
    // At least the message shouldn't be trash then.
    // I don't think "stale" data can be an issue given the memory barriers.
    // Also, stale data is verified during latency tests
    fn verify(&self, _: usize) -> bool {
        for i in self.i {
            if i != self.i[0] {
                log::debug!("data corruption, {i} vs {}", self.i[0]);
                return false;
            }
        }
        true
    }
}
impl Verifyable for ShortMsg {
    fn verify(&self, pos: usize) -> bool {
        pos == self.i
    }
}

// TimeStamped for latency tracking
impl TimeStamped for LongMsg {
    #[inline(always)]
    fn ingestion_t(&self) -> Instant {
        self.t
    }
}

impl TimeStamped for ShortMsg {
    #[inline(always)]
    fn ingestion_t(&self) -> Instant {
        self.t
    }
}
