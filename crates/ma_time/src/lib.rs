use once_cell::sync::OnceCell;
use std::{
    num::ParseIntError,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use web_time::UNIX_EPOCH;
// pub type Instant = quanta::Instant;
pub type Duration = std::time::Duration;
pub type Clock = quanta::Clock;

static GLOBAL_CLOCK: OnceCell<Clock> = OnceCell::new();

#[inline(always)]
fn global_clock() -> &'static Clock {
    GLOBAL_CLOCK.get_or_init(|| Clock::new())
}

// Everything is rdtsc brother
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Instant(pub u64);

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Nanos(pub u64);

impl Nanos {
    pub const MAX: Nanos = Nanos(u64::MAX);
    pub const ZERO: Nanos = Nanos(0);

    pub const fn from_secs(s: u64) -> Self {
        Nanos(s * 1_000_000_000)
    }
    pub fn from_secs_f64(s: f64) -> Self {
        Nanos((s * 1_000_000_000.0).round() as u64)
    }
    pub const fn from_millis(s: u64) -> Self {
        Nanos(s * 1_000_000)
    }
    pub const fn from_micros(s: u64) -> Self {
        Nanos(s * 1_000)
    }
    pub fn as_secs(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }
    pub fn as_millis(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
    pub fn as_micros(&self) -> f64 {
        self.0 as f64 / 1_000.0
    }
    pub fn now() -> Self {
        web_time::SystemTime::now().into()
    }
}

impl Instant {
    #[inline(always)]
    pub fn now() -> Self {
        Instant(rdtscp())
    }
    pub fn elapsed(&self) -> Nanos {
        Nanos(global_clock().delta_as_nanos(self.0, rdtscp()))
    }
}

fn rdtscp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::arch::x86_64::__rdtscp;
        unsafe { __rdtscp(&mut 0u32 as *mut _) }
    }
    #[cfg(target_arch = "wasm32")]
    {
        global_clock().raw()
    }
}

impl std::fmt::Display for Nanos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.0;
        if v < 1000 {
            write!(f, "{}ns", v)
        } else if v < 1_000_000 {
            write!(f, "{}mus", v as f64 / 1000.0)
        } else if v < 1_000_000_000 {
            write!(f, "{}ms", (v / 1000) as f64 / 1000.0)
        } else {
            write!(f, "{}s", (v / 1000000) as f64 / 1000.0)
        }
    }
}

impl Sub for Instant {
    type Output = Nanos;

    fn sub(self, rhs: Instant) -> Nanos {
        Nanos(global_clock().delta_as_nanos(rhs.0, self.0))
    }
}
impl Sub<Nanos> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Nanos) -> Instant {
        let nanos_for_100 = global_clock().delta_as_nanos(0, 100);
        Instant(self.0 - rhs.0 / nanos_for_100 * 100)
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Instant {}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Instant {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

pub trait TimeStamped {
    fn ingestion_t(&self) -> Instant;

    #[inline(always)]
    fn elapsed(&self) -> Nanos {
        self.ingestion_t().elapsed()
    }
}

impl Add for Nanos {
    type Output = Nanos;

    fn add(self, rhs: Nanos) -> Nanos {
        Nanos(self.0.wrapping_add(rhs.0))
    }
}

impl AddAssign for Nanos {
    fn add_assign(&mut self, rhs: Nanos) {
        *self = *self + rhs;
    }
}

impl Sub for Nanos {
    type Output = Nanos;

    fn sub(self, rhs: Nanos) -> Nanos {
        Nanos(self.0.wrapping_sub(rhs.0))
    }
}

impl SubAssign for Nanos {
    fn sub_assign(&mut self, rhs: Nanos) {
        *self = *self - rhs;
    }
}

impl Sub<u64> for Nanos {
    type Output = Nanos;

    fn sub(self, rhs: u64) -> Nanos {
        Nanos(self.0.wrapping_sub(rhs))
    }
}

impl SubAssign<u64> for Nanos {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Mul<u32> for Nanos {
    type Output = Nanos;

    fn mul(self, rhs: u32) -> Nanos {
        Nanos(self.0 * rhs as u64)
    }
}

impl Mul<Nanos> for u32 {
    type Output = Nanos;

    fn mul(self, rhs: Nanos) -> Nanos {
        rhs * self
    }
}

impl MulAssign<u32> for Nanos {
    fn mul_assign(&mut self, rhs: u32) {
        *self = *self * rhs;
    }
}

impl Div<u32> for Nanos {
    type Output = Nanos;

    fn div(self, rhs: u32) -> Nanos {
        Nanos(self.0 / rhs as u64)
    }
}
impl Div<usize> for Nanos {
    type Output = Nanos;

    fn div(self, rhs: usize) -> Nanos {
        Nanos(self.0 / rhs as u64)
    }
}

impl DivAssign<u32> for Nanos {
    fn div_assign(&mut self, rhs: u32) {
        *self = *self / rhs;
    }
}
impl Mul<u64> for Nanos {
    type Output = Nanos;

    fn mul(self, rhs: u64) -> Nanos {
        Nanos(self.0 * rhs as u64)
    }
}

impl Mul<Nanos> for u64 {
    type Output = Nanos;

    fn mul(self, rhs: Nanos) -> Nanos {
        rhs * self
    }
}

impl MulAssign<u64> for Nanos {
    fn mul_assign(&mut self, rhs: u64) {
        *self = *self * rhs;
    }
}

impl Div<u64> for Nanos {
    type Output = Nanos;

    fn div(self, rhs: u64) -> Nanos {
        Nanos(self.0 / rhs as u64)
    }
}

impl DivAssign<u64> for Nanos {
    fn div_assign(&mut self, rhs: u64) {
        *self = *self / rhs;
    }
}

impl PartialEq for Nanos {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Nanos {}

impl PartialOrd for Nanos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Nanos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::iter::Sum for Nanos {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        Nanos(iter.map(|v| v.0).sum())
    }
}
impl<'a> std::iter::Sum<&'a Self> for Nanos {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        Nanos(iter.map(|v| v.0).sum())
    }
}

impl From<u64> for Nanos {
    fn from(value: u64) -> Self {
        Nanos(value)
    }
}
impl From<u128> for Nanos {
    fn from(value: u128) -> Self {
        Nanos(value as u64)
    }
}
impl From<u32> for Nanos {
    fn from(value: u32) -> Self {
        Nanos(value as u64)
    }
}
impl From<i64> for Nanos {
    fn from(value: i64) -> Self {
        Nanos(value as u64)
    }
}
impl From<i32> for Nanos {
    fn from(value: i32) -> Self {
        Nanos(value as u64)
    }
}

impl From<Nanos> for i64 {
    fn from(val: Nanos) -> Self {
        val.0 as i64
    }
}

impl From<web_time::SystemTime> for Nanos {
    fn from(value: web_time::SystemTime) -> Self {
        Nanos(unsafe {
            value
                .duration_since(UNIX_EPOCH)
                .unwrap_unchecked()
                .as_nanos() as u64
        })
    }
}

impl From<Nanos> for std::time::Duration {
    fn from(value: Nanos) -> Self {
        std::time::Duration::from_nanos(value.0)
    }
}

impl FromStr for Nanos {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.contains("ns") {
            s.parse::<u64>().map(|v| Nanos(v))
        } else {
            let len = s.len();
            s[..len - 2].parse::<u64>().map(|v| Nanos(v))
        }
    }
}

#[inline(always)]
pub fn vsync_busy<F, R>(duration: Option<Nanos>, f: F) -> R
where
    F: FnOnce() -> R,
{
    match duration {
        Some(duration) if duration != Nanos(0) => {
            let start_t = Instant::now();
            let out = f();
            while start_t.elapsed() < duration {}
            out
        }
        _ => f(),
    }
}
#[inline(always)]
pub fn vsync<F, R>(duration: Option<Nanos>, f: F) -> R
where
    F: FnOnce() -> R,
{
    match duration {
        Some(duration) if duration != Nanos(0) => {
            let start_t = Instant::now();
            let out = f();
            let el = start_t.elapsed();
            if el < duration {
                std::thread::sleep((duration - el).into())
            }
            out
        }
        _ => f(),
    }
}
#[inline(always)]
pub fn busy_sleep(duration: Option<Nanos>) {
    match duration {
        None => return,
        Some(duration) if duration == Nanos::ZERO => return,
        Some(duration) => {
            let curt = Instant::now();
            while curt.elapsed() < duration {}
        }
    }
}

// pub fn test_system_tune() {
//     unsafe {
//         let n = 100000;
//         //warmup
//         let mut v = Vec::with_capacity(n);
//         let mut c = time::Instant::now();
//         for i in 0..n {
//             let t = time::Instant::now();
//             v.push(t - c);
//             c = t;
//         }
//         v.fill(time::Nanos::ZERO);

//         let mut c = time::Instant::now();
//         for i in 0..n {
//             let t = time::Instant::now();
//             unsafe { *v.get_unchecked_mut(i) = t - c };
//             c = t;
//         }

//         let to_plot: Vec<(f32, f32)> = v
//             .iter()
//             .enumerate()
//             .map(|(i, &p)| (i as f32, p.0 as f32))
//             .collect();
//         log::debug!("RDTSC timing (cycles):");
//         Chart::new_with_y_range(180, 60, 0.0, 100000.0, 10.0, 2500.0)
//             .linecolorplot(
//                 &Shape::Points(&to_plot),
//                 rgb::RGB8 {
//                     r: 0,
//                     g: 100,
//                     b: 255,
//                 },
//             )
//             .nice();
//         Chart::new_with_y_range(180, 60, 0.0, 100000.0, 0.0, 100.0)
//             .linecolorplot(
//                 &Shape::Points(&to_plot),
//                 rgb::RGB8 {
//                     r: 0,
//                     g: 100,
//                     b: 255,
//                 },
//             )
//             .nice();
//     }
// }

// pub fn now_nanos() -> i64 {
//     as_nanos(Instant::now())
// }

// pub fn as_nanos(t: Instant) -> i64 {
//     unsafe{now() - UNIX_EPOCH}
// }

// pub fn from_nanos(nanos: i64) -> SystemTime {
//     UNIX_EPOCH + Duration::from_nanos(nanos as u64)
// }
// pub fn from_micros(micros: i64) -> SystemTime {
//     UNIX_EPOCH + Duration::from_micros(micros as u64)
// }

// #[derive(Debug, Copy, Clone, PartialEq, serde::Serialize)]
// pub struct Timestamp {
//     pub ingestion_t: SystemTime,
//     pub exchange_t: SystemTime,
// }

// impl Timestamp {
//     pub fn from_millis(ingestion_t: SystemTime, millis: u64) -> Self {
//         Self {
//             ingestion_t,
//             exchange_t: UNIX_EPOCH + Duration::from_millis(millis),
//         }
//     }
//     pub fn from_nanos(ingestion_t: SystemTime, nanos: u64) -> Self {
//         Self {
//             ingestion_t,
//             exchange_t: UNIX_EPOCH + Duration::from_nanos(nanos),
//         }
//     }
//     pub fn from_micros(ingestion_t: SystemTime, micros: u64) -> Self {
//         Self {
//             ingestion_t,
//             exchange_t: UNIX_EPOCH + Duration::from_micros(micros),
//         }
//     }

//     pub fn now_and_millis(millis: u64) -> Self {
//         Self::from_millis(now(), millis)
//     }

//     pub fn now_and_nanos(nanos: u64) -> Self {
//         Self::from_nanos(now(), nanos)
//     }

//     pub fn set_exchange_t(&mut self, exchange_t: SystemTime) {
//         self.exchange_t = exchange_t;
//     }
//     pub fn set_ingestion_t(&mut self, ingestion_t: SystemTime) {
//         self.ingestion_t = ingestion_t;
//     }
// }

// impl Default for Timestamp {
//     fn default() -> Self {
//         Self {
//             ingestion_t: UNIX_EPOCH,
//             exchange_t: UNIX_EPOCH
//         }
//     }
// }

// //TODO: make derive
// pub trait Timestamped {
//     fn timestamp(&self) -> Timestamp;
//     fn timestamp_mut(&mut self) -> &mut Timestamp;

//     #[inline(always)]
//     fn ingestion_t(&self) -> SystemTime {
//         self.timestamp().ingestion_t
//     }

//     #[inline(always)]
//     fn exchange_t(&self) -> SystemTime {
//         self.timestamp().exchange_t
//     }

//     #[inline(always)]
//     fn exchange_nanos(&self) -> i64 {
//         as_nanos(self.exchange_t())
//     }

//     #[inline(always)]
//     fn ingestion_nanos(&self) -> i64 {
//         as_nanos(self.ingestion_t())
//     }

//     #[inline(always)]
//     fn exchange_t_mut(&mut self) -> &mut SystemTime {
//         &mut self.timestamp_mut().exchange_t
//     }
// }
//
