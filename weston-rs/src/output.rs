use libc;
use std::os;
use libweston_sys::{
    weston_output,
    weston_output_set_scale, weston_output_set_transform,
    weston_output_enable, weston_output_disable,
    weston_output_release
};
use wayland_sys::server::wl_signal;

pub struct Output {
    ptr: *mut weston_output,
}

impl From<*mut weston_output> for Output {
    fn from(ptr: *mut weston_output) -> Output {
        Output {
            ptr,
        }
    }
}

impl From<*mut os::raw::c_void> for Output {
    fn from(ptr: *mut os::raw::c_void) -> Output {
        Self::from(ptr as *mut weston_output)
    }
}

impl Output {
    pub fn set_scale(&mut self, scale: libc::c_int) {
        unsafe { weston_output_set_scale(self.ptr, scale); }
    }

    pub fn set_transform(&mut self, transform: libc::c_uint) {
        unsafe { weston_output_set_transform(self.ptr, transform); }
    }

    pub fn enable(&mut self) -> bool {
        unsafe { weston_output_enable(self.ptr) == 0 }
    }

    pub fn disable(&mut self) {
        unsafe { weston_output_disable(self.ptr); }
    }

    prop_accessors!(wl_signal | frame_signal, destroy_signal);

    pub fn ptr(&self) -> *mut weston_output {
        self.ptr
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe { weston_output_release(self.ptr); }
    }
}
