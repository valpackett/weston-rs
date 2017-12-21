use libc;
use libweston_sys::{
    weston_output,
    weston_output_set_scale, weston_output_set_extra_scale, weston_output_set_transform,
    weston_output_enable, weston_output_disable,
    weston_output_release
};
use wayland_sys::server::wl_signal;

pub struct Output {
    ptr: *mut weston_output,
    temp: bool,
}

weston_object!(Output << weston_output);

impl Output {

    pub fn set_scale(&self, scale: libc::c_int) {
        unsafe { weston_output_set_scale(self.ptr, scale); }
    }

    pub fn set_extra_scale(&self, scale: libc::c_float) {
        unsafe { weston_output_set_extra_scale(self.ptr, scale); }
    }

    pub fn set_transform(&self, transform: libc::c_uint) {
        unsafe { weston_output_set_transform(self.ptr, transform); }
    }

    pub fn enable(&self) -> bool {
        unsafe { weston_output_enable(self.ptr) == 0 }
    }

    pub fn disable(&self) {
        unsafe { weston_output_disable(self.ptr); }
    }

    prop_accessors!(ptr wl_signal | frame_signal, destroy_signal);
}

impl Drop for Output {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_output_release(self.ptr); }
        }
    }
}
