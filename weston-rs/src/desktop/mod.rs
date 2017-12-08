use std::{marker};
use libweston_sys::{
    weston_desktop, weston_desktop_create, weston_desktop_destroy,
    weston_desktop_api,
};
use ::compositor::Compositor;

pub mod client;
pub mod surface;

pub use self::client::DesktopClient;
pub use self::surface::DesktopSurface;


pub struct Desktop<'comp, UD: 'comp> {
    ptr: *mut weston_desktop,
    phantom: marker::PhantomData<(&'comp Compositor, &'comp UD)>,
}

impl<'comp, UD> Desktop<'comp, UD> {
    pub fn new(compositor: &'comp Compositor, api: &'comp weston_desktop_api, user_data: &'comp UD) -> Desktop<'comp, UD> {
        Desktop {
            ptr: unsafe { weston_desktop_create(compositor.ptr(), api, user_data as *const UD as *mut _) },
            phantom: marker::PhantomData,
        }
    }

    pub fn ptr(&self) -> *mut weston_desktop {
        self.ptr
    }
}

impl<'comp, UD> Drop for Desktop<'comp, UD> {
    fn drop(&mut self) {
        unsafe { weston_desktop_destroy(self.ptr); }
    }
}
