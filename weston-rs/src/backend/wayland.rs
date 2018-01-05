use std::{mem, ptr, ffi, marker};
use libc;
use libweston_sys::{
    weston_backend_config,
    weston_wayland_backend_init, weston_wayland_backend_config,
};
use ::WestonObject;
use ::compositor::Compositor;
use super::Backend;

#[derive(Builder)]
pub struct WaylandBackendConfig {
    #[builder(default)]
    use_pixman: bool,
    #[builder(default)]
    sprawl: bool,
    #[builder(default)]
    display_name: Option<String>,
    #[builder(default)]
    fullscreen: bool,
    #[builder(default = "\"default\".into()")]
    cursor_theme: String,
    #[builder(default = "16")]
    cursor_size: libc::c_int,
}

impl Into<weston_wayland_backend_config> for WaylandBackendConfig {
    fn into(self) -> weston_wayland_backend_config {
        let WaylandBackendConfig { use_pixman, sprawl, display_name, fullscreen, cursor_theme, cursor_size } = self;
        weston_wayland_backend_config {
            base: weston_backend_config {
                struct_version: 2,
                struct_size: mem::size_of::<weston_wayland_backend_config>(),
            },
            use_pixman,
            sprawl,
            display_name: display_name.map(|s| ffi::CString::new(s).expect("CString::new").into_raw()).unwrap_or(ptr::null_mut()),
            fullscreen,
            cursor_theme: ffi::CString::new(cursor_theme).expect("CString::new").into_raw(),
            cursor_size,
        }
    }
}

pub struct WaylandBackend<'comp> {
    id: libc::c_int,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> WaylandBackend<'comp> {
    pub fn new(compositor: &Compositor, config: WaylandBackendConfig) -> WaylandBackend {
        // conf will get memcpy'd by libweston
        let mut config: weston_wayland_backend_config = config.into();
        let id = unsafe { weston_wayland_backend_init(compositor.ptr(), &mut config.base as *mut _) };
        WaylandBackend {
            id,
            phantom: marker::PhantomData,
        }
    }
}

impl<'comp> Backend for WaylandBackend<'comp> {
    fn id(&self) -> libc::c_int {
        self.id
    }
}
