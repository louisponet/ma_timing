use ma_timing::Timer;
use ma_time::Instant;

#[no_mangle]
#[inline(always)]
pub extern "C" fn create_timer(
    name: *const std::os::raw::c_char,
    timer: &mut Timer
)
{
    let p = unsafe{ std::ffi::CStr::from_ptr(name)}.to_str().unwrap();
    let t = Timer::new(p);
    *timer = t;
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn start(
    timer: &mut Timer
)
{
    timer.start();
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn stop(
    timer: &mut Timer
)
{
    timer.stop();
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn latency(
    timer: &mut Timer,
    timestamp: Instant
)
{
    timer.latency(timestamp);
}
