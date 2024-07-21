use ma_time::{Duration, Instant};

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
