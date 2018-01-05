use std::{mem, ptr, ffi, marker, borrow};
use libweston_sys::{
    weston_plugin_api_get,
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
    DRM_OUTPUT_API_NAME = "weston_drm_output_api_v1";
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum DrmBackendOutputMode {
    Off = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_OFF,
    Current = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_CURRENT,
    Preferred = weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_PREFERRED,
}

pub trait DrmOutput {
    fn set_mode(&self, output: &Output, mode: DrmBackendOutputMode, modeline: Option<&str>) -> bool;
    fn set_gbm_format(&self, output: &Output, gbm_format: Option<&str>);
    fn set_seat(&self, output: &Output, seat: Option<&str>);
}

pub struct DrmOutputImpl<'comp> {
    ptr: *mut weston_drm_output_api,
    temp: bool,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> DrmOutput for DrmOutputImpl<'comp> {
    fn set_mode(&self, output: &Output, mode: DrmBackendOutputMode, modeline: Option<&str>) -> bool {
        let modeline = modeline.map(|m| ffi::CString::new(m).expect("CString"));
        unsafe { (*self.ptr).set_mode.expect("set_mode ptr")(output.ptr(), mode as weston_drm_backend_output_mode, modeline.map(|m| m.as_ptr()).unwrap_or(ptr::null())) == 0 }
    }

    fn set_gbm_format(&self, output: &Output, gbm_format: Option<&str>) {
        let gbm_format = gbm_format.map(|f| ffi::CString::new(f).expect("CString"));
        unsafe { (*self.ptr).set_gbm_format.expect("set_gbm_format ptr")(output.ptr(), gbm_format.map(|f| f.as_ptr()).unwrap_or(ptr::null())) }
    }

    fn set_seat(&self, output: &Output, seat: Option<&str>) {
        let seat = seat.map(|s| ffi::CString::new(s).expect("CString"));
        unsafe { (*self.ptr).set_seat.expect("set_gbm_format ptr")(output.ptr(), seat.map(|s| s.as_ptr()).unwrap_or(ptr::null())) }
    }
}

impl<'comp> WestonObject for DrmOutputImpl<'comp> {
    type T = weston_drm_output_api;

    fn from_ptr(ptr: *mut Self::T) -> Self {
        DrmOutputImpl {
            ptr,
            temp: false,
            phantom: marker::PhantomData,
        }
    }

    fn from_ptr_temporary(ptr: *mut Self::T) -> Self {
        DrmOutputImpl {
            ptr,
            temp: true,
            phantom: marker::PhantomData,
        }
    }

    fn ptr(&self) -> *mut Self::T {
        self.ptr
    }
}

pub trait HasDrmOutput<'t> {
    type Impl: DrmOutput;

    fn get_drm_output(&self) -> Option<Self::Impl>;
}

impl<'comp, C: borrow::Borrow<Compositor>> HasDrmOutput<'comp> for C {
    type Impl = DrmOutputImpl<'comp>;

    fn get_drm_output(&self) -> Option<Self::Impl> {
        let ptr = unsafe {
            weston_plugin_api_get(
                self.borrow().ptr(),
                DRM_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_drm_output_api>())
        } as *mut weston_drm_output_api;
        if ptr.is_null() {
            return None
        }
        Some(DrmOutputImpl::from_ptr_temporary(ptr))
    }
}
