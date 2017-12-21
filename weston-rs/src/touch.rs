use libweston_sys::{
    weston_touch, weston_touch_destroy
};
use wayland_sys::server::wl_signal;
use ::seat::Seat;
use ::view::View;

pub struct Touch {
    ptr: *mut weston_touch,
    temp: bool,
}

weston_object!(Touch << weston_touch);

impl Touch {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt View | focus = |&this| { (*this.ptr).focus });
    prop_accessors!(u32 | focus_serial, num_tp, grab_serial);
    prop_accessors!(ptr wl_signal | focus_signal);
}

impl Drop for Touch {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_touch_destroy(self.ptr); }
        }
    }
}
