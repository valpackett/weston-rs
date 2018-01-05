use std::{mem, ptr, marker};
use libc;
use libweston_sys::{
    weston_backend_config,
    weston_drm_backend_init, weston_drm_backend_config,
};
use ::WestonObject;
use ::compositor::Compositor;
use super::Backend;

pub struct DrmBackend<'comp> {
    id: libc::c_int,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> DrmBackend<'comp> {
    pub fn new(compositor: &Compositor, tty: libc::c_int) -> DrmBackend {
        let mut conf = weston_drm_backend_config {
            base: weston_backend_config {
                struct_version: 3,
                struct_size: mem::size_of::<weston_drm_backend_config>(),
            },
            tty,
            use_pixman: false,
            seat_id: ptr::null_mut(),
            gbm_format: ptr::null_mut(),
            configure_device: None,
            pageflip_timeout: 0,
        };
        let id = unsafe { weston_drm_backend_init(
                compositor.ptr(),
                &mut conf.base as *mut _) };
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
