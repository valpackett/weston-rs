use libc;
use libweston_sys::{
    weston_output,
    weston_output_set_scale, weston_output_set_extra_scale, weston_output_set_transform,
    weston_output_enable, weston_output_disable,
    weston_output_release
};
use wayland_sys::server::wl_signal;
use foreign_types::ForeignTypeRef;

foreign_type! {
    type CType = weston_output;
    fn drop = weston_output_release;
    pub struct Output;
    pub struct OutputRef;
}

impl OutputRef {
    prop_accessors!(ptr wl_signal | frame_signal, destroy_signal);

    pub fn set_scale(&mut self, scale: libc::c_int) {
        unsafe { weston_output_set_scale(self.as_ptr(), scale); }
    }

    pub fn set_extra_scale(&mut self, scale: libc::c_float) {
        unsafe { weston_output_set_extra_scale(self.as_ptr(), scale); }
    }

    pub fn set_transform(&mut self, transform: libc::c_uint) {
        unsafe { weston_output_set_transform(self.as_ptr(), transform); }
    }

    pub fn enable(&mut self) -> bool {
        unsafe { weston_output_enable(self.as_ptr()) == 0 }
    }

    pub fn disable(&mut self) {
        unsafe { weston_output_disable(self.as_ptr()); }
    }
}
