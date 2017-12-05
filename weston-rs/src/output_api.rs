use std::{mem, ffi};
use libweston_sys::{
    weston_output,
    weston_plugin_api_get,
    weston_windowed_output_api,
};
use super::compositor::Compositor;

const_cstr!{
    WINDOWED_OUTPUT_API_NAME = "weston_windowed_output_api_v1";
}

pub struct WindowedOutput {
    ptr: *mut weston_windowed_output_api,
}

impl WindowedOutput {
    pub fn new(compositor: &Compositor) -> WindowedOutput {
        let ptr = unsafe {
            weston_plugin_api_get(
                compositor.ptr(),
                WINDOWED_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_windowed_output_api>())
        } as *mut weston_windowed_output_api;
        WindowedOutput {
            ptr: ptr
        }
    }

    pub fn output_create(&self, compositor: &Compositor, name: &str) {
        let name = ffi::CString::new(name).expect("CString");
        unsafe { (*self.ptr).output_create.expect("output_create ptr")(compositor.ptr(), name.as_ptr()) };
    }

    pub fn output_set_size(&self, output: *mut weston_output, width: u32, height: u32) {
        unsafe { (*self.ptr).output_set_size.expect("output_set_size ptr")(output, width as _, height as _) };
    }

    pub fn ptr(&self) -> *mut weston_windowed_output_api {
        self.ptr
    }
}
