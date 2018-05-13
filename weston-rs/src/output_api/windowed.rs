use std::{mem, ffi};
use libweston_sys::{
    weston_plugin_api_get,
    weston_windowed_output_api,
};
use foreign_types::ForeignTypeRef;
use ::compositor::CompositorRef;
use ::output::OutputRef;

const_cstr!{
    WINDOWED_OUTPUT_API_NAME = "weston_windowed_output_api_v1";
}

pub trait WindowedOutput {
    fn output_set_size(&self, output: &OutputRef, width: u32, height: u32) -> bool;
    fn create_head(&self, compositor: &CompositorRef, name: &str) -> bool;
}

// The API impl is not a create/destroy thing really
unsafe fn noop_destroy(_: *mut weston_windowed_output_api) {}

foreign_type! {
    type CType = weston_windowed_output_api;
    fn drop = noop_destroy;
    pub struct WindowedOutputImpl;
    pub struct WindowedOutputImplRef;
}

impl WindowedOutput for WindowedOutputImplRef {
    fn output_set_size(&self, output: &OutputRef, width: u32, height: u32) -> bool {
        unsafe { (*self.as_ptr()).output_set_size.expect("output_set_size ptr")(output.as_ptr(), width as _, height as _) == 0 }
    }

    fn create_head(&self, compositor: &CompositorRef, name: &str) -> bool {
        let name = ffi::CString::new(name).expect("CString");
        unsafe { (*self.as_ptr()).create_head.expect("create_head ptr")(compositor.as_ptr(), name.as_ptr()) == 0 }
    }
}

pub trait HasWindowedOutput {
    type Impl: WindowedOutput;

    fn get_windowed_output(&self) -> Option<&mut Self::Impl>;
}

impl HasWindowedOutput for CompositorRef {
    type Impl = WindowedOutputImplRef;

    fn get_windowed_output(&self) -> Option<&mut Self::Impl> {
        let ptr = unsafe {
            weston_plugin_api_get(
                self.as_ptr(),
                WINDOWED_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_windowed_output_api>())
        } as *mut weston_windowed_output_api;
        if ptr.is_null() {
            return None
        }
        Some(unsafe { WindowedOutputImplRef::from_ptr_mut(ptr) })
    }
}
