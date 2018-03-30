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
use foreign_types::ForeignTypeRef;
use ::seat::SeatRef;
use ::view::ViewRef;

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
    fn focus(&mut self, _pointer: &mut PointerRef) {}
    fn motion(&mut self, _pointer: &mut PointerRef, _time: &libc::timespec, _event: PointerMotionEvent) {}
    fn button(&mut self, _pointer: &mut PointerRef, _time: &libc::timespec, _button: u32, _state: ButtonState) {}
    fn axis(&mut self, _pointer: &mut PointerRef, _time: &libc::timespec, _event: PointerAxisEvent) {}
    fn axis_source(&mut self, _pointer: &mut PointerRef, _source: AxisSource) {}
    fn frame(&mut self, _pointer: &mut PointerRef) {}
    fn cancel(&mut self, _pointer: &mut PointerRef) {}

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
    wrapper.user.focus(unsafe { PointerRef::from_ptr_mut((*grab).pointer) });
}

#[allow(unused_unsafe)]
extern "C" fn run_motion<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, event: *mut weston_pointer_motion_event) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.motion(
        unsafe { PointerRef::from_ptr_mut((*grab).pointer) },
        unsafe { &*time },
        unsafe { (&*event).into() },
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_button<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, button: u32, state: u32) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.button(
        unsafe { PointerRef::from_ptr_mut((*grab).pointer) },
        unsafe { &*time },
        button,
        ButtonState::from_raw(state).unwrap_or(ButtonState::Released),
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_axis<T: PointerGrab>(grab: *mut weston_pointer_grab, time: *const libc::timespec, event: *mut weston_pointer_axis_event) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.axis(
        unsafe { PointerRef::from_ptr_mut((*grab).pointer) },
        unsafe { &*time },
        unsafe { (&*event).into() },
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_axis_source<T: PointerGrab>(grab: *mut weston_pointer_grab, source: u32) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.axis_source(
        unsafe { PointerRef::from_ptr_mut((*grab).pointer) },
        AxisSource::from_raw(source).unwrap_or(AxisSource::Wheel)
    );
}

#[allow(unused_unsafe)]
extern "C" fn run_frame<T: PointerGrab>(grab: *mut weston_pointer_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.frame(unsafe { PointerRef::from_ptr_mut((*grab).pointer) });
}

#[allow(unused_unsafe)]
extern "C" fn run_cancel<T: PointerGrab>(grab: *mut weston_pointer_grab) {
    let wrapper = unsafe { &mut *wl_container_of!(((*grab).interface), PointerGrabWrapper<T>, base) };
    wrapper.user.cancel(unsafe { PointerRef::from_ptr_mut((*grab).pointer) });
}

foreign_type! {
    type CType = weston_pointer;
    fn drop = weston_pointer_destroy;
    pub struct Pointer;
    pub struct PointerRef;
}

impl PointerRef {
    obj_accessors!(SeatRef | seat seat_mut = |&this| { (*this.as_ptr()).seat });
    obj_accessors!(opt ViewRef |
                   focus focus_mut = |&this| { (*this.as_ptr()).focus },
                   sprite sprite_mut = |&this| { (*this.as_ptr()).sprite });
    prop_accessors!(u32 | focus_serial, grab_button, grab_serial, button_count);
    prop_accessors!(i32 | hotspot_x, hotspot_y);
    prop_accessors!(wl_fixed_t | grab_x, grab_y, x, y, sx, sy);
    prop_accessors!(ptr wl_signal | focus_signal, motion_signal, destroy_signal);
    prop_accessors!(weston_pointer_grab | default_grab);

    pub fn motion_to_abs(&self, event: PointerMotionEvent) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_pointer_motion_to_abs(self.as_ptr(), &mut event.into(), &mut x, &mut y); }
        (x, y)
    }

    pub fn send_motion(&mut self, time: &libc::timespec, event: PointerMotionEvent) {
        unsafe { weston_pointer_send_motion(self.as_ptr(), time, &mut event.into()); }
    }

    pub fn has_focus_resource(&self) -> bool {
        unsafe { weston_pointer_has_focus_resource(self.as_ptr()) }
    }

    pub fn send_button(&mut self, time: &libc::timespec, button: u32, state_w: ButtonState) {
        unsafe { weston_pointer_send_button(self.as_ptr(), time, button, state_w.to_raw()); }
    }

    pub fn send_axis(&mut self, time: &libc::timespec, event: PointerAxisEvent) {
        unsafe { weston_pointer_send_axis(self.as_ptr(), time, &mut event.into()); }
    }

    pub fn send_axis_source(&mut self, source: u32) {
        unsafe { weston_pointer_send_axis_source(self.as_ptr(), source); }
    }

    pub fn send_frame(&mut self) {
        unsafe { weston_pointer_send_frame(self.as_ptr()); }
    }

    pub fn set_focus(&mut self, view: &ViewRef, sx: wl_fixed_t, sy: wl_fixed_t) {
        unsafe { weston_pointer_set_focus(self.as_ptr(), view.as_ptr(), sx, sy); }
    }

    pub fn clear_focus(&mut self) {
        unsafe { weston_pointer_clear_focus(self.as_ptr()); }
    }

    pub fn start_grab<T: PointerGrab>(&mut self, grab: T) {
        // XXX: leaks the wrapper
        let silly_wrapper = Box::new(weston_pointer_grab {
            interface: unsafe { grab.into_weston() },
            pointer: self.as_ptr(), // weston will set that to the same value lol
        });
        unsafe { weston_pointer_start_grab(self.as_ptr(), Box::into_raw(silly_wrapper)); }
    }

    pub fn end_grab(&mut self) {
        unsafe { weston_pointer_end_grab(self.as_ptr()); }
    }

    pub fn set_default_grab<T: PointerGrab>(&mut self, grab: T) {
        unsafe { weston_pointer_set_default_grab(self.as_ptr(), grab.into_weston()); }
    }

    pub fn clamp(&self) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_pointer_clamp(self.as_ptr(), &mut x, &mut y); }
        (x, y)
    }

    pub fn moove(&mut self, event: PointerMotionEvent) {
        unsafe { weston_pointer_move(self.as_ptr(), &mut event.into()); }
    }

    pub fn is_default_grab(&self) -> bool {
        return unsafe { (*self.as_ptr()).grab == &mut (*self.as_ptr()).default_grab };
    }
}
