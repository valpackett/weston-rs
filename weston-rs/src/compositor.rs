use std::ptr;
use libc;
use wayland_sys::server::{wl_display};
use libweston_sys::{
    weston_compositor,
    weston_compositor_create, weston_compositor_destroy, weston_compositor_shutdown,
    weston_compositor_set_xkb_rule_names, xkb_rule_names,
    weston_compositor_wake, weston_compositor_schedule_repaint,
    weston_pending_output_coldplug
};

pub struct Compositor {
    ptr: *mut weston_compositor,
}

impl Compositor {
    pub fn new(display: *mut wl_display) -> Compositor {
        let mut result = Compositor {
            ptr: unsafe { weston_compositor_create(display, ptr::null_mut()) },
        };
        // TODO check ptr != null
        unsafe { (*result.ptr).user_data = &mut result as *mut _ as *mut libc::c_void };
        result
    }

    pub fn set_xkb_rule_names(&mut self, names: Option<*mut xkb_rule_names>) {
        unsafe { weston_compositor_set_xkb_rule_names(self.ptr, names.unwrap_or(ptr::null_mut())); }
    }

    pub fn schedule_repaint(&mut self) {
        unsafe { weston_compositor_schedule_repaint(self.ptr); }
    }

    pub fn pending_output_coldplug(&mut self) {
        unsafe { weston_pending_output_coldplug(self.ptr); }
    }

    pub fn wake(&mut self) {
        unsafe { weston_compositor_wake(self.ptr); }
    }

    pub fn shutdown(&mut self) {
        unsafe { weston_compositor_shutdown(self.ptr); }
    }

    pub fn ptr(&self) -> *mut weston_compositor {
        self.ptr
    }
}

impl Drop for Compositor {
    fn drop(&mut self) {
        unsafe { weston_compositor_destroy(self.ptr); }
    }
}
