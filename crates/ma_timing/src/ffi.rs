use std::ffi::{c_char, CStr};

use crate::Timer;

#[no_mangle]
pub extern "C" fn InitTimer(name: *const c_char, dst: *mut Timer<'static>) {
    let t = Timer::new(unsafe { CStr::from_ptr(name).to_str().expect("Can not read string argument.") });
    unsafe{dst.copy_from(&t as *const _, 1)};
    std::mem::forget(t);
}

#[no_mangle]
pub extern "C" fn Start(dst: *mut Timer<'static>) {
    
    let dst_t = unsafe { &mut (*dst)};
    dst_t.start();
}

#[no_mangle]
pub extern "C" fn Stop(dst: *mut Timer<'static>) {
    
    let dst_t = unsafe { &mut (*dst)};
    dst_t.stop();
}
