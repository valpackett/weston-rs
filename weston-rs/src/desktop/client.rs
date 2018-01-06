use libc;
use libweston_sys::{
    weston_desktop_client,
    weston_desktop_client_get_client, weston_desktop_client_for_each_surface,
    weston_desktop_client_ping,
    weston_desktop_surface,
};
use wayland_server;
use ::WestonObject;
use super::surface::DesktopSurface;

pub struct DesktopClient {
    ptr: *mut weston_desktop_client,
    temp: bool,
}

weston_object!(DesktopClient << weston_desktop_client);

impl DesktopClient {
    pub fn get_client(&self) -> wayland_server::Client {
        unsafe { wayland_server::Client::from_ptr(weston_desktop_client_get_client(self.ptr)) }
    }

    pub fn for_each_surface<SC, T: FnMut(DesktopSurface<SC>)>(&self, callback: T) {
        unsafe { weston_desktop_client_for_each_surface(self.ptr, Some(run_callback::<SC, T>), &callback as *const _ as *mut libc::c_void) }
    }

    pub fn ping(&self) -> libc::c_int {
        unsafe { weston_desktop_client_ping(self.ptr) }
    }
}

#[allow(unused_unsafe)]
extern "C" fn run_callback<SC, T: FnMut(DesktopSurface<SC>)>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let cb = unsafe { &mut *(user_data as *mut T) };
    cb(DesktopSurface::from_ptr_temporary(surface));
}
