use std::{mem, ffi};
use libweston_sys::{
    weston_plugin_api_get,
    weston_windowed_output_api,
};
use super::compositor::Compositor;
use super::output::Output;

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
            ptr,
        }
    }

    pub fn output_create(&self, compositor: &Compositor, name: &str) -> bool {
        let name = ffi::CString::new(name).expect("CString");
        unsafe { (*self.ptr).output_create.expect("output_create ptr")(compositor.ptr(), name.as_ptr()) == 0 }
    }

    pub fn output_set_size(&self, output: &Output, width: u32, height: u32) -> bool {
        unsafe { (*self.ptr).output_set_size.expect("output_set_size ptr")(output.ptr(), width as _, height as _) == 0 }
    }

    pub fn ptr(&self) -> *mut weston_windowed_output_api {
        self.ptr
    }
}
