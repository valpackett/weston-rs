use std::{mem, ptr, ffi};
use libc;
use libweston_sys::{
    weston_backend_config,
    weston_wayland_backend_init, weston_wayland_backend_config
};
use super::compositor::Compositor;

pub trait Backend {
    fn id(&self) -> libc::c_int;
}

pub struct WaylandBackend {
    id: libc::c_int,
    // cursor_theme: ffi::CString,
    // conf: weston_wayland_backend_config,
}

impl WaylandBackend {
    pub fn new(compositor: &Compositor) -> WaylandBackend {
        let cursor_theme = ffi::CString::new("Adwaita").unwrap();
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
            id: id,
            // cursor_theme: cursor_theme,
            // conf: conf,
        }
    }
}

impl Backend for WaylandBackend {
    fn id(&self) -> libc::c_int {
        self.id
    }
}
