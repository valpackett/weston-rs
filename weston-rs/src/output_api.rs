use std::{mem, ptr, ffi, marker};
use libweston_sys::{
    weston_plugin_api_get,
    weston_windowed_output_api,
    weston_drm_output_api,
    weston_drm_backend_output_mode,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_OFF,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_CURRENT,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_PREFERRED,
};
use ::WestonObject;
use ::compositor::Compositor;
use ::output::Output;

const_cstr!{
    WINDOWED_OUTPUT_API_NAME = "weston_windowed_output_api_v1";
    DRM_OUTPUT_API_NAME = "weston_drm_output_api_v1";
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

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum DrmBackendOutputMode {
    Off = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_OFF,
    Current = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_CURRENT,
    Preferred = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_PREFERRED,
}

pub struct DrmOutput<'comp> {
    ptr: *mut weston_drm_output_api,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> DrmOutput<'comp> {
    pub fn new(compositor: &'comp Compositor) -> DrmOutput {
        let ptr = unsafe {
            weston_plugin_api_get(
                compositor.ptr(),
                DRM_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_drm_output_api>())
        } as *mut weston_drm_output_api;
        DrmOutput {
            ptr,
            phantom: marker::PhantomData,
        }
    }

    pub fn set_mode(&self, output: &Output, mode: DrmBackendOutputMode, modeline: Option<&str>) -> bool {
        let modeline = modeline.map(|m| ffi::CString::new(m).expect("CString"));
        unsafe { (*self.ptr).set_mode.expect("set_mode ptr")(output.ptr(), mode as weston_drm_backend_output_mode, modeline.map(|m| m.as_ptr()).unwrap_or(ptr::null())) == 0 }
    }

    pub fn set_gbm_format(&self, output: &Output, gbm_format: Option<&str>) {
        let gbm_format = gbm_format.map(|f| ffi::CString::new(f).expect("CString"));
        unsafe { (*self.ptr).set_gbm_format.expect("set_gbm_format ptr")(output.ptr(), gbm_format.map(|f| f.as_ptr()).unwrap_or(ptr::null())) }
    }

    pub fn set_seat(&self, output: &Output, seat: Option<&str>) {
        let seat = seat.map(|s| ffi::CString::new(s).expect("CString"));
        unsafe { (*self.ptr).set_seat.expect("set_gbm_format ptr")(output.ptr(), seat.map(|s| s.as_ptr()).unwrap_or(ptr::null())) }
    }

    pub fn ptr(&self) -> *mut weston_drm_output_api {
        self.ptr
    }
}
