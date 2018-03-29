use libc;
use libweston_sys::{
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_ABS,
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL,
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL_UNACCEL,
    weston_pointer_motion_event, weston_pointer_axis_event,
    weston_pointer_grab, weston_pointer_grab_interface,
    weston_pointer_start_grab, weston_pointer_end_grab,
    weston_pointer_set_default_grab,
    weston_pointer, weston_pointer_destroy,
    weston_pointer_motion_to_abs, weston_pointer_send_motion,
    weston_pointer_has_focus_resource, weston_pointer_send_button,
    weston_pointer_send_axis, weston_pointer_send_axis_source,
    weston_pointer_send_frame,
    weston_pointer_set_focus, weston_pointer_clear_focus,
    weston_pointer_clamp, weston_pointer_move,
};
use wayland_sys::common::wl_fixed_t;
use wayland_sys::server::wl_signal;
pub use wayland_server::protocol::wl_pointer::{Axis, AxisSource, ButtonState};
use ::WestonObject;
use ::seat::Seat;
use ::view::View;

bitflags! {
    #[derive(Default)]
    pub struct PointerMotionMask: u32 {
        const ABS = weston_pointer_motion_mask_WESTON_POINTER_MOTION_ABS;
        const REL = weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL;
        const RELUNACCEL = weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL_UNACCEL;
    }
}

#[derive(Copy, Clone)]
pub struct PointerMotionEvent {
    mask: PointerMotionMask,
    time: libc::timespec,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    dx_unaccel: f64,
    dy_unaccel: f64,
}

impl<'a> From<&'a weston_pointer_motion_event> for PointerMotionEvent {
    fn from(source: &weston_pointer_motion_event) -> Self {
        let &weston_pointer_motion_event { mask, time, x, y, dx, dy, dx_unaccel, dy_unaccel } = source;
        PointerMotionEvent {
            mask: PointerMotionMask::from_bits_truncate(mask), time, x, y, dx, dy, dx_unaccel, dy_unaccel
        }
    }
}

impl Into<weston_pointer_motion_event> for PointerMotionEvent {
    fn into(self) -> weston_pointer_motion_event {
        let PointerMotionEvent { mask, time, x, y, dx, dy, dx_unaccel, dy_unaccel } = self;
        weston_pointer_motion_event {
            mask: mask.bits(), time, x, y, dx, dy, dx_unaccel, dy_unaccel
        }
    }
}

#[derive(Copy, Clone)]
pub struct PointerAxisEvent {
    axis: Axis,
    value: f64,
    has_discrete: bool,
    discrete: i32,

}
impl<'a> From<&'a weston_pointer_axis_event> for PointerAxisEvent {
    fn from(source: &weston_pointer_axis_event) -> Self {
        let &weston_pointer_axis_event { axis, value, has_discrete, discrete } = source;
        PointerAxisEvent {
            axis: Axis::from_raw(axis).unwrap_or(Axis::VerticalScroll), value, has_discrete, discrete
        }
    }
}

impl Into<weston_pointer_axis_event> for PointerAxisEvent {
    fn into(self) -> weston_pointer_axis_event {
        let PointerAxisEvent { axis, value, has_discrete, discrete } = self;
        weston_pointer_axis_event {
            axis: axis.to_raw(), value, has_discrete, discrete
        }
    }
}

pub trait PointerGrab where Self: Sized {
    fn focus(&mut self, pointer: &mut Pointer) {}
    fn motion(&mut self, pointer: &mut Pointer, time: &libc::timespec, event: PointerMotionEvent) {}
    fn button(&mut self, pointer: &mut Pointer, time: &libc::timespec, button: u32, state: ButtonState) {}
    fn axis(&mut self, pointer: &mut Pointer, time: &libc::timespec, event: PointerAxisEvent) {}
    fn axis_source(&mut self, pointer: &mut Pointer, source: AxisSource) {}
    fn frame(&mut self, pointer: &mut Pointer) {}
    fn cancel(&mut self, pointer: &mut Pointer) {}

    unsafe fn into_weston(self) -> *mut weston_pointer_grab_interface {
        let wrapper = Box::new(PointerGrabWrapper {
            base: weston_pointer_grab_interface {
                focus: Some(run_focus::<Self>),
                motion: Some(run_motion::<Self>),
                button: Some(run_button::<Self>),
                axis: Some(run_axis::<Self>),
                axis_source: Some(run_axis_source::<Self>),
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
struct PointerGrabWrapper<T: PointerGrab> {
    base: weston_pointer_grab_interface,
    user: T,
}

#[allow(unused_unsafe)]
extern "C" fn run_focus<T: PointerGrab>(grab: *mut weston_pointer_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.focus(&mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }));
}

#[allow(unused_unsafe)]
extern "C" fn run_motion<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, event: *mut weston_pointer_motion_event) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.motion(
        &mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }),
        unsafe { &*time },
        unsafe { (&*event).into() },
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_button<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, button: u32, state: u32) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.button(
        &mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }),
        unsafe { &*time },
        button,
        ButtonState::from_raw(state).unwrap_or(ButtonState::Released),
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_axis<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, event: *mut weston_pointer_axis_event) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.axis(
        &mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }),
        unsafe { &*time },
        unsafe { (&*event).into() },
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_axis_source<T: PointerGrab>(grab: *mut weston_pointer_grab, source: u32) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.axis_source(
        &mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }),
        AxisSource::from_raw(source).unwrap_or(AxisSource::Wheel)
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_frame<T: PointerGrab>(grab: *mut weston_pointer_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.frame(&mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }));
}

#[allow(unused_unsafe)]
extern "C" fn run_cancel<T: PointerGrab>(grab: *mut weston_pointer_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.cancel(&mut Pointer::from_ptr_temporary(unsafe { (*grab).pointer }));
}

pub struct Pointer {
    ptr: *mut weston_pointer,
    temp: bool,
}

weston_object!(Pointer << weston_pointer);

impl Pointer {
    obj_accessors!(Seat | seat = |&this| { (*this.ptr).seat });
    obj_accessors!(opt View |
                   focus = |&this| { (*this.ptr).focus },
                   sprite = |&this| { (*this.ptr).sprite });
    prop_accessors!(u32 | focus_serial, grab_button, grab_serial, button_count);
    prop_accessors!(i32 | hotspot_x, hotspot_y);
    prop_accessors!(wl_fixed_t | grab_x, grab_y, x, y, sx, sy);
    prop_accessors!(ptr wl_signal | focus_signal, motion_signal, destroy_signal);
    prop_accessors!(weston_pointer_grab | default_grab);

    pub fn motion_to_abs(&self, event: PointerMotionEvent) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_pointer_motion_to_abs(self.ptr, &mut event.into(), &mut x, &mut y); }
        (x, y)
    }

    pub fn send_motion(&self, time: &libc::timespec, event: PointerMotionEvent) {
        unsafe { weston_pointer_send_motion(self.ptr, time, &mut event.into()); }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_pointer_has_focus_resource(self.ptr) }
    }

    pub fn send_button(&self, time: &libc::timespec, button: u32, state_w: ButtonState) {
        unsafe { weston_pointer_send_button(self.ptr, time, button, state_w.to_raw()); }
    }

    pub fn send_axis(&self, time: &libc::timespec, event: PointerAxisEvent) {
        unsafe { weston_pointer_send_axis(self.ptr, time, &mut event.into()); }
    }

    pub fn send_axis_source(&self, source: u32) {
        unsafe { weston_pointer_send_axis_source(self.ptr, source); }
    }

    pub fn send_frame(&self) {
        unsafe { weston_pointer_send_frame(self.ptr); }
    }

    pub fn set_focus(&self, view: &View, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_pointer_set_focus(self.ptr, view.ptr(), sx, sy); }
    }

    pub fn clear_focus(&self) {
        unsafe { weston_pointer_clear_focus(self.ptr); }
    }

    pub fn start_grab<T: PointerGrab>(&self, grab: T) {
        // XXX: leaks the wrapper
        let silly_wrapper = Box::new(weston_pointer_grab {
            interface: unsafe { grab.into_weston() },
            pointer: self.ptr, // weston will set that to the same value lol
        });
        unsafe { weston_pointer_start_grab(self.ptr, Box::into_raw(silly_wrapper)); }
    }

    pub fn end_grab(&self) {
        unsafe { weston_pointer_end_grab(self.ptr); }
    }

    pub fn set_default_grab<T: PointerGrab>(&self, grab: T) {
        unsafe { weston_pointer_set_default_grab(self.ptr, grab.into_weston()); }
    }

    pub fn clamp(&self) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_pointer_clamp(self.ptr, &mut x, &mut y); }
        (x, y)
    }

    pub fn moove(&self, event: PointerMotionEvent) {
        unsafe { weston_pointer_move(self.ptr, &mut event.into()); }
    }

    pub fn is_default_grab(&self) -> bool {
        return unsafe { (*self.ptr).grab == &mut (*self.ptr).default_grab };
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_pointer_destroy(self.ptr); }
        }
    }
}
