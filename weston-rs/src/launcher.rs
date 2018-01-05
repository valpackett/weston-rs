use libc;
use std::mem;
use std::ffi::CStr;
use std::os::unix::io::RawFd;
use libweston_sys::{
    launcher_interface,
    weston_launcher,
    weston_compositor,
};
use ::WestonObject;
use ::compositor::Compositor;

pub trait Launcher where Self: Sized {
    fn connect(compositor: &Compositor, tty: libc::c_int, seat_id: &CStr, sync_drm: bool) -> Option<Self>;
    fn open(&mut self, path: &CStr, flags: libc::c_int) -> RawFd;
    fn close(&mut self, fd: RawFd);
    fn activate_vt(&mut self, vt: libc::c_int) -> bool;
    fn restore(&mut self);
    fn get_vt(&mut self) -> libc::c_int;

    unsafe fn into_weston(self) -> *mut weston_launcher {
        let mut wrapper = Box::new(LauncherWrapper {
            base: mem::zeroed(),
            user: self,
        });
        let iface = Box::new(launcher_interface {
            connect: Some(run_connect::<Self>),
            destroy: Some(run_destroy::<Self>),
            open: Some(run_open::<Self>),
            close: Some(run_close::<Self>),
            activate_vt: Some(run_activate_vt::<Self>),
            restore: Some(run_restore::<Self>),
            get_vt: Some(run_get_vt::<Self>),
        });
        wrapper.base.iface = Box::into_raw(iface);
        let raw = Box::into_raw(wrapper);
        &mut (*raw).base
    }
}

#[repr(C)]
struct LauncherWrapper<T: Launcher> {
    base: weston_launcher,
    user: T,
}

#[allow(unused_unsafe)]
pub extern "C" fn run_connect<T: Launcher>(
    launcher_out: *mut *mut weston_launcher,
    compositor: *mut weston_compositor,
    tty: libc::c_int,
    seat_id: *const libc::c_char,
    sync_drm: bool) -> libc::c_int {
    if let Some(launcher) = T::connect(&Compositor::from_ptr_temporary(compositor), tty, unsafe { CStr::from_ptr(seat_id) }, sync_drm) {
        unsafe { *launcher_out = launcher.into_weston() };
        0
    } else {
        -1
    }
}

#[allow(unused_unsafe)]
pub extern "C" fn run_destroy<T: Launcher>(launcher: *mut weston_launcher) {
    let wrapper = unsafe { Box::from_raw(wl_container_of!(launcher, LauncherWrapper<T>, base)) };
    unsafe {
        let iface_ptr = mem::transmute::<*const launcher_interface, *mut launcher_interface>(wrapper.base.iface);
        Box::from_raw(iface_ptr);
    }
}

#[allow(unused_unsafe)]
pub extern "C" fn run_open<T: Launcher>(launcher: *mut weston_launcher, path: *const libc::c_char, flags: libc::c_int) -> libc::c_int {
    let wrapper = unsafe { &mut *wl_container_of!(launcher, LauncherWrapper<T>, base) };
    wrapper.user.open(unsafe { CStr::from_ptr(path) }, flags)
}

#[allow(unused_unsafe)]
pub extern "C" fn run_close<T: Launcher>(launcher: *mut weston_launcher, fd: libc::c_int) {
    let wrapper = unsafe { &mut *wl_container_of!(launcher, LauncherWrapper<T>, base) };
    wrapper.user.close(fd);
}

#[allow(unused_unsafe)]
pub extern "C" fn run_activate_vt<T: Launcher>(launcher: *mut weston_launcher, vt: libc::c_int) -> libc::c_int {
    let wrapper = unsafe { &mut *wl_container_of!(launcher, LauncherWrapper<T>, base) };
    wrapper.user.activate_vt(vt) as libc::c_int
}

#[allow(unused_unsafe)]
pub extern "C" fn run_restore<T: Launcher>(launcher: *mut weston_launcher) {
    let wrapper = unsafe { &mut *wl_container_of!(launcher, LauncherWrapper<T>, base) };
    wrapper.user.restore();
}

#[allow(unused_unsafe)]
pub extern "C" fn run_get_vt<T: Launcher>(launcher: *mut weston_launcher) -> libc::c_int {
    let wrapper = unsafe { &mut *wl_container_of!(launcher, LauncherWrapper<T>, base) };
    wrapper.user.get_vt()
}
