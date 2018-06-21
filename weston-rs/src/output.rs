use std::ptr;
use libc;
use libweston_sys::{
    weston_output,
    weston_output_set_scale, weston_output_set_extra_scale, weston_output_set_transform,
    weston_output_enable, weston_output_disable, weston_output_destroy,
    weston_output_iterate_heads, weston_head,
};
use wayland_sys::server::wl_signal;
use foreign_types::ForeignTypeRef;
use ::head::HeadRef;

foreign_type! {
    type CType = weston_output;
    fn drop = weston_output_destroy;
    pub struct Output;
    pub struct OutputRef;
}

pub struct HeadIterator<'a> {
    output: &'a OutputRef,
    head: *mut weston_head,
}

impl<'a> Iterator for HeadIterator<'a> {
    type Item = &'a mut HeadRef;

    fn next(&mut self) -> Option<&'a mut HeadRef> {
        self.head = unsafe { weston_output_iterate_heads(self.output.as_ptr(), self.head) };
        if self.head.is_null() {
            None
        } else {
            Some(unsafe { HeadRef::from_ptr_mut(self.head) })
        }
    }
}

impl OutputRef {
    prop_accessors!(i32 | x, y, width, height, native_scale, current_scale, original_scale);
    prop_accessors!(libc::c_int | scale);
    prop_accessors!(f32 | extra_scale, current_extra_scale);
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

    pub fn iterate_heads(&mut self) -> HeadIterator {
        HeadIterator {
            output: self,
            head: ptr::null_mut(),
        }
    }
}
