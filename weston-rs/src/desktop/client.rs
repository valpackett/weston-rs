use std::mem;
use libc;
use libweston_sys::{
    weston_desktop_client,
    weston_desktop_client_get_client, weston_desktop_client_for_each_surface,
    weston_desktop_client_ping,
    weston_desktop_surface,
};
use wayland_server;
use foreign_types::{ForeignType, ForeignTypeRef};
use super::surface::DesktopSurfaceRef;

// The desktop_client is not a create/destroy thing really
fn noop_destroy(_: *mut weston_desktop_client) {}

foreign_type! {
    type CType = weston_desktop_client;
    fn drop = noop_destroy;
    pub struct DesktopClient;
    pub struct DesktopClientRef;
}

impl DesktopClientRef {
    pub fn get_client(&self) -> wayland_server::Client {
        unsafe { wayland_server::Client::from_ptr(weston_desktop_client_get_client(self.as_ptr())) }
    }

    pub fn for_each_surface<SC, T: FnMut(&mut DesktopSurfaceRef<SC>)>(&self, callback: T) {
        unsafe { weston_desktop_client_for_each_surface(self.as_ptr(), Some(run_callback::<SC, T>), &callback as *const _ as *mut libc::c_void) }
    }

    pub fn ping(&self) -> libc::c_int {
        unsafe { weston_desktop_client_ping(self.as_ptr()) }
    }
}

#[allow(unused_unsafe)]
extern "C" fn run_callback<SC, T: FnMut(&mut DesktopSurfaceRef<SC>)>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let cb = unsafe { &mut *(user_data as *mut T) };
    cb(unsafe { DesktopSurfaceRef::from_ptr_mut(surface) });
}
