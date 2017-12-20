use std::os;
use libweston_sys::{
    weston_pointer, weston_pointer_destroy
};
use wayland_sys::server::wl_signal;
use ::seat::Seat;
use ::view::View;

pub struct Pointer {
    ptr: *mut weston_pointer,
}

impl From<*mut weston_pointer> for Pointer {
    fn from(ptr: *mut weston_pointer) -> Pointer {
        Pointer {
            ptr,
        }
    }
}

impl From<*mut os::raw::c_void> for Pointer {
    fn from(ptr: *mut os::raw::c_void) -> Pointer {
        Self::from(ptr as *mut weston_pointer)
    }
}

impl Pointer {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt View |
                   focus = |&this| { (*this.ptr).focus },
                   sprite = |&this| { (*this.ptr).sprite });
    prop_accessors!(u32 | focus_serial, grab_button, grab_serial, button_count);
    prop_accessors!(i32 | hotspot_x, hotspot_y);
    prop_accessors!(wl_signal | focus_signal, motion_signal, destroy_signal);

    pub fn ptr(&self) -> *mut weston_pointer {
        self.ptr
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        unsafe { weston_pointer_destroy(self.ptr); }
    }
}
