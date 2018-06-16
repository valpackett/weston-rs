#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate libc;
extern crate wayland_sys;
extern crate input_sys;
extern crate xkbcommon;

use libc::timespec;
use wayland_sys::common::*;
// use wayland_sys::client::*;
use wayland_sys::server::*;
// use wayland_sys::server::wl_display; // disambiguate. Both are opaque actually, users can transmute if needed
// use wayland_sys::cursor::*;
// use wayland_sys::egl::*;
use input_sys::libinput_device;
use xkbcommon::xkb::ffi::*;

// these got blacklisted by the wl_.* regex and whitelist doesn't override :(
pub type wl_data_device_manager_dnd_action = libc::c_uint;
pub type wl_pointer_button_state = libc::c_uint;
pub type wl_keyboard_key_state = libc::c_uint;
pub type wl_output_transform = libc::c_uint;
pub type wl_output_subpixel = libc::c_uint;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(all(feature = "logind", target_os = "linux"))]
extern "C" {
    pub static launcher_logind_iface: launcher_interface;
}
