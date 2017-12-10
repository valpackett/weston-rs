use std::mem;
use std::os::raw::c_void;
use wayland_sys::server::{signal, wl_signal, wl_listener};

#[macro_export]
macro_rules! wl_container_of {
    ($ptr:expr, $type:ty, $member:ident) => {{
        ($ptr as *mut u8).offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }}
}

pub struct WlListener<T> {
    cb: Box<FnMut(&mut T)>,
    wll: wl_listener,
}

#[allow(unused_unsafe)]
extern "C" fn run_wl_listener<T: From<*mut c_void>>(listener: *mut wl_listener, data: *mut c_void) {
    let wrapper = unsafe { &mut *wl_container_of!(listener, WlListener<T>, wll) };
    let mut obj = mem::ManuallyDrop::new(data.into());
    (*wrapper.cb)(&mut obj);
}

impl<T: From<*mut c_void>> WlListener<T> {
    pub fn new(cb: Box<FnMut(&mut T)>) -> Box<WlListener<T>> {
        let mut result = Box::new(WlListener {
            cb,
            wll: unsafe { mem::zeroed() },
        });
        result.wll.notify = run_wl_listener::<T>;
        result
    }

    pub fn signal_add(&mut self, signal: &mut wl_signal) {
        unsafe { signal::wl_signal_add(signal, &mut self.wll); }
    }
}
