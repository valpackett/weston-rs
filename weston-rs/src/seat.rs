use std::os;
use libweston_sys::{
    weston_seat,
    weston_seat_release,
    weston_seat_get_pointer, weston_seat_get_touch
};
use wayland_sys::server::wl_signal;
use ::pointer::Pointer;
use ::touch::Touch;

pub struct Seat {
    ptr: *mut weston_seat,
}

impl From<*mut weston_seat> for Seat {
    fn from(ptr: *mut weston_seat) -> Seat {
        Seat {
            ptr,
        }
    }
}

impl From<*mut os::raw::c_void> for Seat {
    fn from(ptr: *mut os::raw::c_void) -> Seat {
        Self::from(ptr as *mut weston_seat)
    }
}

impl Seat {
    obj_accessors!(opt Pointer | get_pointer = |&this| { weston_seat_get_pointer(this.ptr) });
    obj_accessors!(opt Touch | get_touch = |&this| { weston_seat_get_touch(this.ptr) });
    prop_accessors!(wl_signal | destroy_signal, updated_caps_signal, selection_signal);

    pub fn ptr(&self) -> *mut weston_seat {
        self.ptr
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        unsafe { weston_seat_release(self.ptr); }
    }
}
