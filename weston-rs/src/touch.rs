use libc;
use libweston_sys::{
    weston_touch_grab, weston_touch_grab_interface,
    weston_touch_start_grab, weston_touch_end_grab,
    weston_touch, weston_touch_destroy,
    weston_touch_set_focus, weston_touch_has_focus_resource,
    weston_touch_send_down, weston_touch_send_up,
    weston_touch_send_motion, weston_touch_send_frame,
};
use wayland_sys::common::wl_fixed_t;
use wayland_sys::server::wl_signal;
use ::WestonObject;
use ::seat::Seat;
use ::view::View;

pub trait TouchGrab where Self: Sized {
    fn down(&mut self, touch: &mut Touch, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {}
    fn up(&mut self, touch: &mut Touch, time: &libc::timespec, touch_id: libc::c_int) {}
    fn motion(&mut self, touch: &mut Touch, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {}
    fn frame(&mut self, touch: &mut Touch);
    fn cancel(&mut self, touch: &mut Touch);

    unsafe fn into_weston(self) -> *mut weston_touch_grab_interface {
        let mut wrapper = Box::new(TouchGrabWrapper {
            base: weston_touch_grab_interface {
                down: Some(run_down::<Self>),
                up: Some(run_up::<Self>),
                motion: Some(run_motion::<Self>),
                frame: Some(run_frame::<Self>),
                cancel: Some(run_cancel::<Self>),
            },
            user: self,
        });
        let raw = Box::into_raw(wrapper);
        &mut (*raw).base
    }
}

#[repr(C)]
struct TouchGrabWrapper<T: TouchGrab> {
    base: weston_touch_grab_interface,
    user: T,
}

#[allow(unused_unsafe)]
pub extern "C" fn run_down<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.down(&mut Touch::from_ptr_temporary(unsafe { (*grab).touch }), unsafe { &*time }, touch_id, sx, sy);
}

#[allow(unused_unsafe)]
pub extern "C" fn run_up<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.up(&mut Touch::from_ptr_temporary(unsafe { (*grab).touch }), unsafe { &*time }, touch_id);
}

#[allow(unused_unsafe)]
pub extern "C" fn run_motion<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.motion(&mut Touch::from_ptr_temporary(unsafe { (*grab).touch }), unsafe { &*time }, touch_id, sx, sy);
}

#[allow(unused_unsafe)]
pub extern "C" fn run_frame<T: TouchGrab>(grab: *mut weston_touch_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.frame(&mut Touch::from_ptr_temporary(unsafe { (*grab).touch }));
}

#[allow(unused_unsafe)]
pub extern "C" fn run_cancel<T: TouchGrab>(grab: *mut weston_touch_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.cancel(&mut Touch::from_ptr_temporary(unsafe { (*grab).touch }));
}

pub struct Touch {
    ptr: *mut weston_touch,
    temp: bool,
}

weston_object!(Touch << weston_touch);

impl Touch {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt View | focus = |&this| { (*this.ptr).focus });
    prop_accessors!(u32 | focus_serial, num_tp, grab_serial);
    prop_accessors!(ptr wl_signal | focus_signal);

    pub fn set_focus(&self, view: &View) {
        unsafe { weston_touch_set_focus(self.ptr, view.ptr()); }
    }

    pub fn start_grab<T: TouchGrab>(&self, grab: T) {
        // XXX: leaks the wrapper
        let silly_wrapper = Box::new(weston_touch_grab {
            interface: unsafe { grab.into_weston() },
            touch: self.ptr, // weston will set that to the same value lol
        });
        unsafe { weston_touch_start_grab(self.ptr, Box::into_raw(silly_wrapper)); }
    }

    pub fn end_grab(&self) {
        unsafe { weston_touch_end_grab(self.ptr); }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_touch_has_focus_resource(self.ptr) }
    }

    pub fn send_down(&self, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_touch_send_down(self.ptr, time, touch_id, sx, sy); }
    }

    pub fn send_up(&self, time: &libc::timespec, touch_id: libc::c_int) {
        unsafe { weston_touch_send_up(self.ptr, time, touch_id); }
    }

    pub fn send_motion(&self, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_touch_send_motion(self.ptr, time, touch_id, sx, sy); }
    }

    pub fn send_frame(&self) {
        unsafe { weston_touch_send_frame(self.ptr); }
    }
}

impl Drop for Touch {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_touch_destroy(self.ptr); }
        }
    }
}
