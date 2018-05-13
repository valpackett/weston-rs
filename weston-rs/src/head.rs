use std::ffi;
use libweston_sys::{
    weston_head,
    weston_head_get_name, weston_head_get_output,
    weston_head_is_connected, weston_head_is_enabled,
    weston_head_is_device_changed, weston_head_reset_device_changed,
    weston_head_detach,
};
use foreign_types::ForeignTypeRef;
use ::output::{Output, OutputRef};

unsafe fn noop_destroy(_: *mut weston_head) {}

foreign_type! {
    type CType = weston_head;
    fn drop = noop_destroy;
    pub struct Head;
    pub struct HeadRef;
}

impl HeadRef {
    obj_accessors!(opt OutputRef |
                   output output_mut = |&this| { weston_head_get_output(this.as_ptr()) });
    obj_accessors!(opt Output |
                   output_owned = |&this| { weston_head_get_output(this.as_ptr()) });

    pub fn get_name(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(weston_head_get_name(self.as_ptr())) }
    }

    pub fn is_connected(&self) -> bool {
        unsafe { weston_head_is_connected(self.as_ptr()) }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { weston_head_is_enabled(self.as_ptr()) }
    }

    pub fn is_device_changed(&self) -> bool {
        unsafe { weston_head_is_device_changed(self.as_ptr()) }
    }

    pub fn reset_device_changed(&mut self) {
        unsafe { weston_head_reset_device_changed(self.as_ptr()); }
    }

    pub fn detach(&mut self) {
        unsafe { weston_head_detach(self.as_ptr()); }
    }

}
