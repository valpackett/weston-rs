use std::os;
use libweston_sys::{
    weston_touch, weston_touch_destroy
};
use wayland_sys::server::wl_signal;
use ::seat::Seat;
use ::view::View;

pub struct Touch {
    ptr: *mut weston_touch,
}

impl From<*mut weston_touch> for Touch {
    fn from(ptr: *mut weston_touch) -> Touch {
        Touch {
            ptr,
        }
    }
}

impl From<*mut os::raw::c_void> for Touch {
    fn from(ptr: *mut os::raw::c_void) -> Touch {
        Self::from(ptr as *mut weston_touch)
    }
}

impl Touch {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt View | focus = |&this| { (*this.ptr).focus });
    prop_accessors!(u32 | focus_serial, num_tp, grab_serial);
    prop_accessors!(wl_signal | focus_signal);

    pub fn ptr(&self) -> *mut weston_touch {
        self.ptr
    }
}

impl Drop for Touch {
    fn drop(&mut self) {
        unsafe { weston_touch_destroy(self.ptr); }
    }
}
