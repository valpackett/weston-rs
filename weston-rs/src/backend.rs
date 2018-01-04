use std::{mem, ptr, ffi, marker};
use libc;
use libweston_sys::{
    weston_backend_config,
    weston_wayland_backend_init, weston_wayland_backend_config,
    weston_drm_backend_init, weston_drm_backend_config,
};
use ::WestonObject;
use ::compositor::Compositor;

pub trait Backend {
    fn id(&self) -> libc::c_int;
}

pub struct WaylandBackend<'comp> {
    id: libc::c_int,
    _cursor_theme: ffi::CString,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> WaylandBackend<'comp> {
    pub fn new(compositor: &Compositor) -> WaylandBackend {
        let cursor_theme = ffi::CString::new("default").unwrap();
        // conf will get memcpy'd by libweston
        let mut conf = weston_wayland_backend_config {
            base: weston_backend_config {
                struct_version: 2,
                struct_size: mem::size_of::<weston_wayland_backend_config>(),
            },
            use_pixman: false,
            sprawl: false,
            display_name: ptr::null_mut(),
            fullscreen: false,
            cursor_theme: cursor_theme.as_ptr() as *mut _,
            cursor_size: 16,
        };
        let id = unsafe { weston_wayland_backend_init(
                compositor.ptr(),
                &mut conf.base as *mut _) };
        WaylandBackend {
            id,
            _cursor_theme: cursor_theme,
            phantom: marker::PhantomData,
        }
    }
}

impl<'comp> Backend for WaylandBackend<'comp> {
    fn id(&self) -> libc::c_int {
        self.id
    }
}

pub struct DrmBackend<'comp> {
    id: libc::c_int,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> DrmBackend<'comp> {
    pub fn new(compositor: &Compositor, tty: libc::c_int) -> DrmBackend {
        let mut conf = weston_drm_backend_config {
            base: weston_backend_config {
                struct_version: 3,
                struct_size: mem::size_of::<weston_wayland_backend_config>(),
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
