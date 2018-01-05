use std::{mem, ptr, ffi, marker};
use libc;
use libweston_sys::{
    weston_backend_config,
    weston_drm_backend_init, weston_drm_backend_config,
    weston_compositor,
};
use input_sys::libinput_device;
use ::WestonObject;
use ::compositor::Compositor;
use super::Backend;

#[derive(Builder)]
pub struct DrmBackendConfig {
    #[builder(default)]
    tty: libc::c_int,
    #[builder(default)]
    use_pixman: bool,
    #[builder(default)]
    seat_id: Option<String>,
    #[builder(default)]
    gbm_format: Option<String>,
    #[builder(default)]
    configure_device: Option<unsafe extern "C" fn(*mut weston_compositor, *mut libinput_device)>,
    #[builder(default)]
    pageflip_timeout: u32,
}

impl Into<weston_drm_backend_config> for DrmBackendConfig {
    fn into(self) -> weston_drm_backend_config {
        let DrmBackendConfig { tty, use_pixman, seat_id, gbm_format, configure_device, pageflip_timeout } = self;
        weston_drm_backend_config {
            base: weston_backend_config {
                struct_version: 3,
                struct_size: mem::size_of::<weston_drm_backend_config>(),
            },
            tty,
            use_pixman,
            // these accept NULL as default
            seat_id: seat_id.map(|s| ffi::CString::new(s).expect("CString::new").into_raw()).unwrap_or(ptr::null_mut()),
            gbm_format: gbm_format.map(|s| ffi::CString::new(s).expect("CString::new").into_raw()).unwrap_or(ptr::null_mut()),
            configure_device,
            pageflip_timeout,
        }
    }
}

pub struct DrmBackend<'comp> {
    id: libc::c_int,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> DrmBackend<'comp> {
    pub fn new(compositor: &Compositor, config: DrmBackendConfig) -> DrmBackend {
        // conf will get memcpy'd by libweston
        let mut config: weston_drm_backend_config = config.into();
        let id = unsafe { weston_drm_backend_init(compositor.ptr(), &mut config.base as *mut _) };
        DrmBackend {
            id,
            phantom: marker::PhantomData,
        }
    }
}

impl<'comp> Backend for DrmBackend<'comp> {
    fn id(&self) -> libc::c_int {
        self.id
    }
}
