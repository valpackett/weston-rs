use libc;
use std::{marker};
use libweston_sys::{
    weston_desktop, weston_desktop_create, weston_desktop_destroy,
    weston_desktop_api,
};
use ::compositor::Compositor;

pub mod api;
pub mod client;
pub mod surface;

pub use self::api::DesktopApi;
pub use self::client::DesktopClient;
pub use self::surface::DesktopSurface;


pub struct Desktop<'comp, SC: 'comp> {
    ptr: *mut weston_desktop,
    wapi: Box<weston_desktop_api>,
    api: Box<Box<DesktopApi<SC>>>, // heard you like boxes :D
    // but the outer one gets turned into a raw pointer and we get the inner one in callbacks
    phantom: marker::PhantomData<(&'comp Compositor, SC)>,
}

impl<'comp, SC> Desktop<'comp, SC> {
    pub fn new(compositor: &'comp Compositor, api: Box<DesktopApi<SC>>) -> Desktop<'comp, SC> {
        let mut wapi = self::api::make_weston_api::<SC>();
        let mut api = Box::new(api);
        Desktop {
            ptr: unsafe { weston_desktop_create(compositor.ptr(), &mut *wapi, &mut *api as *mut _ as *mut libc::c_void) },
            wapi,
            api,
            phantom: marker::PhantomData,
        }
    }

    pub fn ptr(&self) -> *mut weston_desktop {
        self.ptr
    }
}

impl<'comp, SC> Drop for Desktop<'comp, SC> {
    fn drop(&mut self) {
        unsafe { weston_desktop_destroy(self.ptr); }
    }
}
