use std::ffi;
use wayland_sys::server::*;
use wayland_server;

pub struct Display {
    ptr: *mut wl_display,
}

unsafe impl Sync for Display {}

impl Display {
    pub fn new() -> Display {
        Display {
            ptr: unsafe { wl_display_create() },
        }
    }

    pub fn from_ptr(ptr: *mut wl_display) -> Display {
        Display {
            ptr
        }
    }

    pub fn add_socket_auto(&mut self) -> ffi::CString {
        unsafe { ffi::CStr::from_ptr(wl_display_add_socket_auto(self.ptr)).to_owned() }
    }

    pub fn run(&self) {
        unsafe { wl_display_run(self.ptr); }
    }

    pub fn get_event_loop(&self) -> wayland_server::EventLoop {
        unsafe { wayland_server::create_event_loop(wl_display_get_event_loop(self.ptr), Some(self.ptr)) }
    }

    pub fn as_ptr(&self) -> *mut wl_display {
        self.ptr
    }
}
