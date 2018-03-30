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
use foreign_types::ForeignTypeRef;
use ::seat::SeatRef;
use ::view::ViewRef;

pub trait TouchGrab where Self: Sized {
    fn down(&mut self, _touch: &mut TouchRef, _time: &libc::timespec, _touch_id: libc::c_int, _sx: wl_fixed_t, _sy: wl_fixed_t) {}
    fn up(&mut self, _touch: &mut TouchRef, _time: &libc::timespec, _touch_id: libc::c_int) {}
    fn motion(&mut self, _touch: &mut TouchRef, _time: &libc::timespec, _touch_id: libc::c_int, _sx: wl_fixed_t, _sy: wl_fixed_t) {}
    fn frame(&mut self, _touch: &mut TouchRef);
    fn cancel(&mut self, _touch: &mut TouchRef);

    unsafe fn into_weston(self) -> *mut weston_touch_grab_interface {
        let wrapper = Box::new(TouchGrabWrapper {
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
extern "C" fn run_down<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.down(unsafe { TouchRef::from_ptr_mut((*grab).touch) }, unsafe { &*time }, touch_id, sx, sy);
}

#[allow(unused_unsafe)]
extern "C" fn run_up<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.up(unsafe { TouchRef::from_ptr_mut((*grab).touch) }, unsafe { &*time }, touch_id);
}

#[allow(unused_unsafe)]
extern "C" fn run_motion<T: TouchGrab>(grab: *mut weston_touch_grab, time: *const libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.motion(unsafe { TouchRef::from_ptr_mut((*grab).touch) }, unsafe { &*time }, touch_id, sx, sy);
}

#[allow(unused_unsafe)]
extern "C" fn run_frame<T: TouchGrab>(grab: *mut weston_touch_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.frame(unsafe { TouchRef::from_ptr_mut((*grab).touch) });
}

#[allow(unused_unsafe)]
extern "C" fn run_cancel<T: TouchGrab>(grab: *mut weston_touch_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), TouchGrabWrapper<T>, base) };
    wrapper.user.cancel(unsafe { TouchRef::from_ptr_mut((*grab).touch) });
}

foreign_type! {
    type CType = weston_touch;
    fn drop = weston_touch_destroy;
    pub struct Touch;
    pub struct TouchRef;
}

impl TouchRef {
    obj_accessors!(SeatRef | seat seat_mut = |&this| { (*this.as_ptr()).seat });
    obj_accessors!(opt ViewRef | focus focus_mut = |&this| { (*this.as_ptr()).focus });
    prop_accessors!(u32 | focus_serial, num_tp, grab_serial);
    prop_accessors!(ptr wl_signal | focus_signal);

    pub fn set_focus(&mut self, view: &ViewRef) {
        unsafe { weston_touch_set_focus(self.as_ptr(), view.as_ptr()); }
    }

    pub fn start_grab<T: TouchGrab>(&mut self, grab: T) {
        // XXX: leaks the wrapper
        let silly_wrapper = Box::new(weston_touch_grab {
            interface: unsafe { grab.into_weston() },
            touch: self.as_ptr(), // weston will set that to the same value lol
        });
        unsafe { weston_touch_start_grab(self.as_ptr(), Box::into_raw(silly_wrapper)); }
    }

    pub fn end_grab(&mut self) {
        unsafe { weston_touch_end_grab(self.as_ptr()); }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_touch_has_focus_resource(self.as_ptr()) }
    }

    pub fn send_down(&mut self, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_touch_send_down(self.as_ptr(), time, touch_id, sx, sy); }
    }

    pub fn send_up(&mut self, time: &libc::timespec, touch_id: libc::c_int) {
        unsafe { weston_touch_send_up(self.as_ptr(), time, touch_id); }
    }

    pub fn send_motion(&mut self, time: &libc::timespec, touch_id: libc::c_int, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_touch_send_motion(self.as_ptr(), time, touch_id, sx, sy); }
    }

    pub fn send_frame(&mut self) {
        unsafe { weston_touch_send_frame(self.as_ptr()); }
    }
}
