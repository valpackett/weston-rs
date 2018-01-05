use libc;
use libweston_sys::{
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_ABS,
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL,
    weston_pointer_motion_mask_WESTON_POINTER_MOTION_REL_UNACCEL,
    weston_pointer_motion_event, weston_pointer_axis_event,
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

impl Into<weston_pointer_motion_event> for PointerMotionEvent {
    fn into(self) -> weston_pointer_motion_event {
        let PointerMotionEvent { mask, time, x, y, dx, dy, dx_unaccel, dy_unaccel } = self;
        weston_pointer_motion_event {
            mask: mask.bits(), time, x, y, dx, dy, dx_unaccel, dy_unaccel
        }
    }
}

pub type PointerAxisEvent = weston_pointer_axis_event;

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
    prop_accessors!(ptr wl_signal | focus_signal, motion_signal, destroy_signal);

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

    pub fn send_button(&self, time: &libc::timespec, button: u32, state_w: u32) {
        unsafe { weston_pointer_send_button(self.ptr, time, button, state_w); }
    }

    pub fn send_axis(&self, time: &libc::timespec, mut event: PointerAxisEvent) {
        unsafe { weston_pointer_send_axis(self.ptr, time, &mut event); }
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

    // TODO grab

    pub fn clamp(&self) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_pointer_clamp(self.ptr, &mut x, &mut y); }
        (x, y)
    }

    pub fn moove(&self, event: PointerMotionEvent) {
        unsafe { weston_pointer_move(self.ptr, &mut event.into()); }
    }
}

impl Drop for Pointer {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_pointer_destroy(self.ptr); }
        }
    }
}
