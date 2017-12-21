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
    temp: bool,
}

weston_object!(Seat << weston_seat);

impl Seat {
    obj_accessors!(opt Pointer | get_pointer = |&this| { weston_seat_get_pointer(this.ptr) });
    obj_accessors!(opt Touch | get_touch = |&this| { weston_seat_get_touch(this.ptr) });
    prop_accessors!(ptr wl_signal | destroy_signal, updated_caps_signal, selection_signal);
}

impl Drop for Seat {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_seat_release(self.ptr); }
        }
    }
}
