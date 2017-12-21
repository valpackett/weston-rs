use std::{mem, marker};
use std::os::raw::c_void;
use wayland_sys::server::{signal, wl_signal, wl_listener};
use ::WestonObject;

#[macro_export]
macro_rules! wl_container_of {
    ($ptr:expr, $type:ty, $member:ident) => {{
        ($ptr as *mut u8).offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }}
}

pub struct WlListener<P, T: WestonObject<P>> {
    cb: Box<FnMut(T)>,
    wll: wl_listener,
    phantom: marker::PhantomData<P>,
}

#[allow(unused_unsafe)]
extern "C" fn run_wl_listener<P, T: WestonObject<P>>(listener: *mut wl_listener, data: *mut c_void) {
    let wrapper = unsafe { &mut *wl_container_of!(listener, WlListener<P, T>, wll) };
    (*wrapper.cb)(T::from_void_ptr_temporary(data));
}

impl<P, T: WestonObject<P>> WlListener<P, T> {
    pub fn new(cb: Box<FnMut(T)>) -> Box<WlListener<P, T>> {
        let mut result = Box::new(WlListener {
            cb,
            wll: unsafe { mem::zeroed() },
            phantom: marker::PhantomData::<P>,
        });
        result.wll.notify = run_wl_listener::<P, T>;
        result
    }

    pub fn signal_add(&mut self, signal: &mut wl_signal) {
        unsafe { signal::wl_signal_add(signal, &mut self.wll); }
    }
}
