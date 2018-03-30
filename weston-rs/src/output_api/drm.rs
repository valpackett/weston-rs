#![allow(non_upper_case_globals)]

use std::{mem, ptr, ffi};
use libweston_sys::{
    weston_plugin_api_get,
    weston_drm_output_api,
    weston_drm_backend_output_mode,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_OFF,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_CURRENT,
    weston_drm_backend_output_mode_WESTON_DRM_BACKEND_OUTPUT_PREFERRED,
};
use foreign_types::ForeignTypeRef;
use ::compositor::CompositorRef;
use ::output::OutputRef;

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
    fn set_mode(&self, output: &OutputRef, mode: DrmBackendOutputMode, modeline: Option<&str>) -> bool;
    fn set_gbm_format(&self, output: &OutputRef, gbm_format: Option<&str>);
    fn set_seat(&self, output: &OutputRef, seat: Option<&str>);
}

// The API impl is not a create/destroy thing really
unsafe fn noop_destroy(_: *mut weston_drm_output_api) {}

foreign_type! {
    type CType = weston_drm_output_api;
    fn drop = noop_destroy;
    pub struct DrmOutputImpl;
    pub struct DrmOutputImplRef;
}

impl DrmOutput for DrmOutputImplRef {
    fn set_mode(&self, output: &OutputRef, mode: DrmBackendOutputMode, modeline: Option<&str>) -> bool {
        let modeline = modeline.map(|m| ffi::CString::new(m).expect("CString"));
        unsafe { (*self.as_ptr()).set_mode.expect("set_mode ptr")(output.as_ptr(), mode as weston_drm_backend_output_mode, modeline.map(|m| m.as_ptr()).unwrap_or(ptr::null())) == 0 }
    }

    fn set_gbm_format(&self, output: &OutputRef, gbm_format: Option<&str>) {
        let gbm_format = gbm_format.map(|f| ffi::CString::new(f).expect("CString"));
        unsafe { (*self.as_ptr()).set_gbm_format.expect("set_gbm_format ptr")(output.as_ptr(), gbm_format.map(|f| f.as_ptr()).unwrap_or(ptr::null())) }
    }

    fn set_seat(&self, output: &OutputRef, seat: Option<&str>) {
        let seat = seat.map(|s| ffi::CString::new(s).expect("CString"));
        unsafe { (*self.as_ptr()).set_seat.expect("set_gbm_format ptr")(output.as_ptr(), seat.map(|s| s.as_ptr()).unwrap_or(ptr::null())) }
    }
}

pub trait HasDrmOutput {
    type Impl: DrmOutput;

    fn get_drm_output(&self) -> Option<&mut Self::Impl>;
}

impl HasDrmOutput for CompositorRef {
    type Impl = DrmOutputImplRef;

    fn get_drm_output(&self) -> Option<&mut Self::Impl> {
        let ptr = unsafe {
            weston_plugin_api_get(
                self.as_ptr(),
                DRM_OUTPUT_API_NAME.as_ptr(),
                mem::size_of::<weston_drm_output_api>())
        } as *mut weston_drm_output_api;
        if ptr.is_null() {
            return None
        }
        Some(unsafe { DrmOutputImplRef::from_ptr_mut(ptr) })
    }
}
