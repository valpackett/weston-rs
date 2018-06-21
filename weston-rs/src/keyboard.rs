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
    weston_keyboard_grab, weston_keyboard_grab_interface,
    weston_keyboard_start_grab, weston_keyboard_end_grab,
    weston_keyboard,
    weston_keyboard_set_focus, weston_keyboard_set_locks,
    weston_keyboard_has_focus_resource, weston_keyboard_send_key,
    weston_keyboard_send_modifiers,
};
use wayland_sys::server::wl_signal;
pub use wayland_server::protocol::wl_keyboard::KeyState;
use foreign_types::ForeignTypeRef;
use ::seat::SeatRef;
use ::surface::SurfaceRef;

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

pub trait KeyboardGrab where Self: Sized {
    fn key(&mut self, _keyboard: &mut KeyboardRef, _time: &libc::timespec, _key: u32, _state: KeyState) {}
    fn modifiers(&mut self, _keyboard: &mut KeyboardRef, _serial: u32,
                 _mods_depressed: KeyboardModifier, _mods_latched: KeyboardModifier,
                 _mods_locked: KeyboardModifier, _group: u32);
    fn cancel(&mut self, _keyboard: &mut KeyboardRef);

    unsafe fn into_weston(self) -> *mut weston_keyboard_grab_interface {
        let wrapper = Box::new(KeyboardGrabWrapper {
            base: weston_keyboard_grab_interface {
                key: Some(run_key::<Self>),
                modifiers: Some(run_modifiers::<Self>),
                cancel: Some(run_cancel::<Self>),
            },
            user: self,
        });
        let raw = Box::into_raw(wrapper);
        &mut (*raw).base
    }
}

#[repr(C)]
struct KeyboardGrabWrapper<T: KeyboardGrab> {
    base: weston_keyboard_grab_interface,
    user: T,
}

unsafe extern "C" fn run_key<T: KeyboardGrab>(grab: *mut weston_keyboard_grab, time: *const libc::timespec, key: u32, state: u32) {
    let wrapper = &mut *wl_container_of!(((*grab).interface), KeyboardGrabWrapper<T>, base);
    wrapper.user.key(
        KeyboardRef::from_ptr_mut((*grab).keyboard),
        &*time,
        key,
        KeyState::from_raw(state).unwrap_or(KeyState::Released)
    );
}

unsafe extern "C" fn run_modifiers<T: KeyboardGrab>(grab: *mut weston_keyboard_grab, serial: u32,
                                                    mods_depressed: u32, mods_latched: u32, mods_locked: u32, group: u32) {
    let wrapper = &mut *wl_container_of!(((*grab).interface), KeyboardGrabWrapper<T>, base);
    wrapper.user.modifiers(
        KeyboardRef::from_ptr_mut((*grab).keyboard),
        serial,
        KeyboardModifier::from_bits_truncate(mods_depressed),
        KeyboardModifier::from_bits_truncate(mods_latched),
        KeyboardModifier::from_bits_truncate(mods_locked),
        group,
    );
}

unsafe extern "C" fn run_cancel<T: KeyboardGrab>(grab: *mut weston_keyboard_grab) {
    let wrapper = &mut *wl_container_of!(((*grab).interface), KeyboardGrabWrapper<T>, base);
    wrapper.user.cancel(KeyboardRef::from_ptr_mut((*grab).keyboard));
}

unsafe fn noop_destroy(_: *mut weston_keyboard) {}

foreign_type! {
    type CType = weston_keyboard;
    fn drop = noop_destroy;
    pub struct Keyboard;
    pub struct KeyboardRef;
}

impl KeyboardRef {
    obj_accessors!(SeatRef | seat seat_mut = |&this| { (*this.as_ptr()).seat });
    obj_accessors!(opt SurfaceRef | focus focus_mut = |&this| { (*this.as_ptr()).focus });
    prop_accessors!(u32 | focus_serial, grab_key, grab_serial);
    prop_accessors!(ptr wl_signal | focus_signal);

    pub fn set_focus(&mut self, surface: &SurfaceRef) {
        unsafe { weston_keyboard_set_focus(self.as_ptr(), surface.as_ptr()); }
    }

    pub fn set_locks(&mut self, mask: KeyboardLock, value: KeyboardLock) -> bool {
        unsafe { weston_keyboard_set_locks(self.as_ptr(), mask.bits(), value.bits()) == 0 }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_keyboard_has_focus_resource(self.as_ptr()) }
    }

    pub fn send_key(&mut self, time: &libc::timespec, key: u32, state: KeyState) {
        unsafe { weston_keyboard_send_key(self.as_ptr(), time, key, state.to_raw()); }
    }

    pub fn send_modifiers(&mut self, serial: u32, mods_depressed: KeyboardModifier,
                      mods_latched: KeyboardModifier, mods_locked: KeyboardModifier, group: u32) {
        unsafe { weston_keyboard_send_modifiers(self.as_ptr(), serial, mods_depressed.bits(), mods_latched.bits(), mods_locked.bits(), group); }
    }

    pub fn start_grab<T: KeyboardGrab>(&mut self, grab: T) {
        // XXX: leaks the wrapper
        let silly_wrapper = Box::new(weston_keyboard_grab {
            interface: unsafe { grab.into_weston() },
            keyboard: self.as_ptr(), // weston will set that to the same value lol
        });
        unsafe { weston_keyboard_start_grab(self.as_ptr(), Box::into_raw(silly_wrapper)); }
    }

    pub fn end_grab(&mut self) {
        unsafe { weston_keyboard_end_grab(self.as_ptr()); }
    }
}
