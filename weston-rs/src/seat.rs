use libc;
use libweston_sys::{
    weston_seat,
    weston_seat_release,
    weston_seat_get_pointer, weston_seat_get_keyboard, weston_seat_get_touch,
    weston_seat_set_keyboard_focus,
};
use wayland_sys::server::wl_signal;
use ::WestonObject;
use ::pointer::Pointer;
use ::keyboard::{KeyboardModifier, Keyboard};
use ::touch::Touch;
use ::surface::Surface;

pub struct Seat {
    ptr: *mut weston_seat,
    temp: bool,
}

weston_object!(Seat << weston_seat);

impl Seat {
    obj_accessors!(opt Pointer | get_pointer = |&this| { weston_seat_get_pointer(this.ptr) });
    obj_accessors!(opt Keyboard | get_keyboard = |&this| { weston_seat_get_keyboard(this.ptr) });
    obj_accessors!(opt Touch | get_touch = |&this| { weston_seat_get_touch(this.ptr) });
    obj_accessors!(opt Surface | saved_kbd_focus = |&this| { (*this.ptr).saved_kbd_focus });
    prop_accessors!(ptr wl_signal | destroy_signal, updated_caps_signal, selection_signal);
    prop_accessors!(libc::c_int | pointer_device_count, keyboard_device_count, touch_device_count);

    pub fn modifier_state(&self) -> KeyboardModifier {
        KeyboardModifier::from_bits_truncate(unsafe { (*self.ptr).modifier_state })
    }

    pub fn set_keyboard_focus(&self, surface: &Surface) {
        unsafe { weston_seat_set_keyboard_focus(self.ptr, surface.ptr()); }
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_seat_release(self.ptr); }
        }
    }
}
