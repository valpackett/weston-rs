use libc;
use libweston_sys::{
    weston_seat,
    weston_seat_release,
    weston_seat_get_pointer, weston_seat_get_keyboard, weston_seat_get_touch,
    weston_seat_set_keyboard_focus,
};
use wayland_sys::server::wl_signal;
use foreign_types::{ForeignType, ForeignTypeRef};
use ::pointer::PointerRef;
use ::keyboard::{KeyboardModifier, KeyboardRef};
use ::touch::TouchRef;
use ::surface::SurfaceRef;

foreign_type! {
    type CType = weston_seat;
    fn drop = weston_seat_release;
    pub struct Seat;
    pub struct SeatRef;
}

impl SeatRef {
    obj_accessors!(opt PointerRef | get_pointer = |&this| { weston_seat_get_pointer(this.as_ptr()) });
    obj_accessors!(opt KeyboardRef | get_keyboard = |&this| { weston_seat_get_keyboard(this.as_ptr()) });
    obj_accessors!(opt TouchRef | get_touch = |&this| { weston_seat_get_touch(this.as_ptr()) });
    obj_accessors!(opt SurfaceRef | saved_kbd_focus = |&this| { (*this.as_ptr()).saved_kbd_focus });
    prop_accessors!(ptr wl_signal | destroy_signal, updated_caps_signal, selection_signal);
    prop_accessors!(libc::c_int | pointer_device_count, keyboard_device_count, touch_device_count);

    pub fn modifier_state(&self) -> KeyboardModifier {
        KeyboardModifier::from_bits_truncate(unsafe { (*self.as_ptr()).modifier_state })
    }

    pub fn set_keyboard_focus(&self, surface: &SurfaceRef) {
        unsafe { weston_seat_set_keyboard_focus(self.as_ptr(), surface.as_ptr()); }
    }
}
