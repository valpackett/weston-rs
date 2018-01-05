use std::{mem, ffi, marker, borrow};
use libweston_sys::{
    weston_plugin_api_get,
    weston_windowed_output_api,
};
use ::WestonObject;
use ::compositor::Compositor;
use ::output::Output;

const_cstr!{
    WINDOWED_OUTPUT_API_NAME = "weston_windowed_output_api_v1";
}

pub trait WindowedOutput<'comp> {
    fn output_set_size(&self, output: &Output, width: u32, height: u32) -> bool;
    fn output_create(&self, compositor: &'comp Compositor, name: &str) -> bool;
}

pub struct WindowedOutputImpl<'comp> {
    ptr: *mut weston_windowed_output_api,
    temp: bool,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> WindowedOutput<'comp> for WindowedOutputImpl<'comp> {
    fn output_set_size(&self, output: &Output, width: u32, height: u32) -> bool {
        unsafe { (*self.ptr).output_set_size.expect("output_set_size ptr")(output.ptr(), width as _, height as _) == 0 }
    }

    fn output_create(&self, compositor: &'comp Compositor, name: &str) -> bool {
        let name = ffi::CString::new(name).expect("CString");
        unsafe { (*self.ptr).output_create.expect("output_create ptr")(compositor.ptr(), name.as_ptr()) == 0 }
    }
}

impl<'comp> WestonObject for WindowedOutputImpl<'comp> {
    type T = weston_windowed_output_api;

    fn from_ptr(ptr: *mut Self::T) -> Self {
        WindowedOutputImpl {
            ptr,
            temp: false,
            phantom: marker::PhantomData,
        }
    }

    fn from_ptr_temporary(ptr: *mut Self::T) -> Self {
        WindowedOutputImpl {
            ptr,
            temp: true,
            phantom: marker::PhantomData,
        }
    }

    fn ptr(&self) -> *mut Self::T {
        self.ptr
    }
}

pub trait HasWindowedOutput<'t> {
    type Impl: WindowedOutput<'t>;

    fn get_windowed_output(&'t self) -> Option<Self::Impl>;
}

impl<'comp, C: borrow::Borrow<Compositor>> HasWindowedOutput<'comp> for C {
    type Impl = WindowedOutputImpl<'comp>;

    fn get_windowed_output(&self) -> Option<Self::Impl> {
        let ptr = unsafe {
            weston_plugin_api_get(
                self.borrow().ptr(),
                WINDOWED_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_windowed_output_api>())
        } as *mut weston_windowed_output_api;
        if ptr.is_null() {
            return None
        }
        Some(WindowedOutputImpl::from_ptr_temporary(ptr))
    }
}
