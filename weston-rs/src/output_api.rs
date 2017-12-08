use std::{mem, ffi, marker};
use libweston_sys::{
    weston_plugin_api_get,
    weston_windowed_output_api,
};
use ::compositor::Compositor;
use ::output::Output;

const_cstr!{
    WINDOWED_OUTPUT_API_NAME = "weston_windowed_output_api_v1";
}

pub struct WindowedOutput<'comp> {
    ptr: *mut weston_windowed_output_api,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> WindowedOutput<'comp> {
    pub fn new(compositor: &'comp Compositor) -> WindowedOutput {
        let ptr = unsafe {
            weston_plugin_api_get(
                compositor.ptr(),
                WINDOWED_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_windowed_output_api>())
        } as *mut weston_windowed_output_api;
        WindowedOutput {
            ptr,
            phantom: marker::PhantomData,
        }
    }

    pub fn output_create(&self, compositor: &'comp Compositor, name: &str) -> bool {
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
