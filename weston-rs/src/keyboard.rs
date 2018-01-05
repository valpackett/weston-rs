use libc;
use libweston_sys::{
    weston_keyboard_modifier_MODIFIER_CTRL,
    weston_keyboard_modifier_MODIFIER_ALT,
    weston_keyboard_modifier_MODIFIER_SUPER,
    weston_keyboard_modifier_MODIFIER_SHIFT,
    weston_keyboard_locks_WESTON_NUM_LOCK,
    weston_keyboard_locks_WESTON_CAPS_LOCK,
    weston_led_LED_NUM_LOCK,
    weston_led_LED_CAPS_LOCK,
    weston_led_LED_SCROLL_LOCK,
    weston_keyboard, weston_keyboard_destroy,
    weston_keyboard_set_focus, weston_keyboard_set_locks,
    weston_keyboard_has_focus_resource, weston_keyboard_send_key,
    weston_keyboard_send_modifiers,
};
use wayland_sys::server::wl_signal;
pub use wayland_server::protocol::wl_keyboard::KeyState;
use ::WestonObject;
use ::seat::Seat;
use ::surface::Surface;

bitflags! {
    #[derive(Default)]
    pub struct KeyboardModifier: u32 {
        const CTRL = weston_keyboard_modifier_MODIFIER_CTRL;
        const ALT = weston_keyboard_modifier_MODIFIER_ALT;
        const SUPER = weston_keyboard_modifier_MODIFIER_SUPER;
        const SHIFT = weston_keyboard_modifier_MODIFIER_SHIFT;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct KeyboardLock: u32 {
        const NUM = weston_keyboard_locks_WESTON_NUM_LOCK;
        const CAPS = weston_keyboard_locks_WESTON_CAPS_LOCK;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct KeyboardLed: u32 {
        const NUM = weston_led_LED_NUM_LOCK;
        const CAPS = weston_led_LED_CAPS_LOCK;
        const SCROLL = weston_led_LED_SCROLL_LOCK;
    }
}

pub struct Keyboard {
    ptr: *mut weston_keyboard,
    temp: bool,
}

weston_object!(Keyboard << weston_keyboard);

impl Keyboard {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt Surface | focus = |&this| { (*this.ptr).focus });
    prop_accessors!(u32 | focus_serial, grab_key, grab_serial);
    prop_accessors!(ptr wl_signal | focus_signal);

    pub fn set_focus(&self, surface: &Surface) {
        unsafe { weston_keyboard_set_focus(self.ptr, surface.ptr()); }
    }

    pub fn set_locks(&self, mask: KeyboardLock, value: KeyboardLock) -> bool {
        unsafe { weston_keyboard_set_locks(self.ptr, mask.bits(), value.bits()) == 0 }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_keyboard_has_focus_resource(self.ptr) }
    }

    pub fn send_key(&self, time: &libc::timespec, key: u32, state: KeyState) {
        unsafe { weston_keyboard_send_key(self.ptr, time, key, state.to_raw()); }
    }

    pub fn send_modifiers(&self, serial: u32, mods_depressed: KeyboardModifier,
                      mods_latched: KeyboardModifier, mods_locked: KeyboardModifier, group: u32) {
        unsafe { weston_keyboard_send_modifiers(self.ptr, serial, mods_depressed.bits(), mods_latched.bits(), mods_locked.bits(), group); }
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_keyboard_destroy(self.ptr); }
        }
    }
}
