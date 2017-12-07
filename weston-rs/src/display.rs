use std::ffi;
use wayland_sys::server::*;

pub struct Display {
    ptr: *mut wl_display,
}

impl Display {
    pub fn new() -> Display {
        Display {
            ptr: unsafe { wl_display_create() },
        }
    }

    pub fn add_socket_auto(&self) -> ffi::CString {
        unsafe { ffi::CStr::from_ptr(wl_display_add_socket_auto(self.ptr)).to_owned() }
    }

    pub fn run(&self) {
        unsafe { wl_display_run(self.ptr); }
    }

    pub fn ptr(&self) -> *mut wl_display {
        self.ptr
    }
}
