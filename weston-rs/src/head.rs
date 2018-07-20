use std::ffi;
use libweston_sys::{
    weston_head,
    weston_head_get_name, weston_head_get_output,
    weston_head_is_connected, weston_head_is_enabled,
    weston_head_is_device_changed, weston_head_reset_device_changed,
    weston_head_set_subpixel, weston_head_set_connection_status, weston_head_set_internal,
    weston_head_set_monitor_strings, weston_head_set_physical_size,
    weston_head_detach, weston_head_release,
};
use wayland_sys::{
    server::wl_signal,
    common::wl_list,
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
    prop_accessors!(i32 | mm_width, mm_height);
    prop_accessors!(u32 | subpixel);
    prop_accessors!(bool | connection_internal, device_changed, connected);
    prop_accessors!(ptr wl_signal | destroy_signal);
    prop_accessors!(ptr wl_list | resource_list); // wl_output protocol objects

    pub fn set_monitor_strings(&mut self, make: &str, model: &str, serialno: &str) {
        let make = ffi::CString::new(make).expect("CString");
        let model = ffi::CString::new(model).expect("CString");
        let serialno = ffi::CString::new(serialno).expect("CString");
        unsafe { weston_head_set_monitor_strings(self.as_ptr(), make.as_ptr(), model.as_ptr(), serialno.as_ptr()); }
    }

    pub fn set_physical_size(&mut self, mm_width: i32, mm_height: i32) {
        unsafe { weston_head_set_physical_size(self.as_ptr(), mm_width, mm_height); }
    }

    pub fn set_subpixel(&mut self, sp: u32) {
        unsafe { weston_head_set_subpixel(self.as_ptr(), sp as _); }
    }

    pub fn set_connection_status(&mut self, connected: bool) {
        unsafe { weston_head_set_connection_status(self.as_ptr(), connected); }
    }

    pub fn set_internal(&mut self) {
        unsafe { weston_head_set_internal(self.as_ptr()); }
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

    pub fn get_name(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(weston_head_get_name(self.as_ptr())) }
    }

    pub fn get_make(&self) -> Option<&ffi::CStr> {
        let make = unsafe { (*self.as_ptr()).make };
        if make.is_null() {
            return None
        }
        Some(unsafe { ffi::CStr::from_ptr(make) })
    }

    pub fn get_model(&self) -> Option<&ffi::CStr> {
        let model = unsafe { (*self.as_ptr()).model };
        if model.is_null() {
            return None
        }
        Some(unsafe { ffi::CStr::from_ptr(model) })
    }

    pub fn get_serial_number(&self) -> Option<&ffi::CStr> {
        let serial_number = unsafe { (*self.as_ptr()).serial_number };
        if serial_number.is_null() {
            return None
        }
        Some(unsafe { ffi::CStr::from_ptr(serial_number) })
    }

    pub fn detach(&mut self) {
        unsafe { weston_head_detach(self.as_ptr()); }
    }

    pub fn release(&mut self) {
        unsafe { weston_head_release(self.as_ptr()); }
    }

}
