use libc;
use std::{marker};
use libweston_sys::{
    weston_desktop, weston_desktop_create, weston_desktop_destroy,
    weston_desktop_api,
};
use foreign_types::{ForeignType, ForeignTypeRef};
use ::compositor::CompositorRef;

pub mod api;
pub mod client;
pub mod surface;

pub use self::api::DesktopApi;
pub use self::client::{DesktopClient, DesktopClientRef};
pub use self::surface::DesktopSurfaceRef;


pub struct Desktop<'comp, SC> {
    ptr: *mut weston_desktop,
    wapi: Box<weston_desktop_api>,
    api: Box<Box<DesktopApi<SC>>>, // heard you like boxes :D
    // but the outer one gets turned into a raw pointer and we get the inner one in callbacks
    phantom: marker::PhantomData<(&'comp CompositorRef, SC)>,
}

impl<'comp, SC> Desktop<'comp, SC> {
    pub fn new(compositor: &'comp CompositorRef, api: Box<DesktopApi<SC>>) -> Desktop<'comp, SC> {
        let wapi = self::api::make_weston_api::<SC>();
        let mut api = Box::new(api);
        Desktop {
            ptr: unsafe { weston_desktop_create(compositor.as_ptr(), &*wapi, &mut *api as *mut _ as *mut libc::c_void) },
            wapi,
            api,
            phantom: marker::PhantomData,
        }
    }

    pub fn api(&'comp mut self) -> &'comp mut DesktopApi<SC> {
        use std::ops::DerefMut;
        self.api.deref_mut().deref_mut()
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
