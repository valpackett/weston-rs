use libc;
use libweston_sys::{
    weston_view, weston_view_create, weston_view_destroy,
    weston_view_set_position, weston_view_set_transform_parent,
    weston_view_set_mask, weston_view_set_mask_infinite,
    weston_view_is_mapped, weston_view_schedule_repaint,
    weston_view_damage_below, weston_view_unmap,
    weston_view_update_transform,
    weston_view_to_global_fixed, weston_view_to_global_float,
    weston_view_from_global_float, weston_view_from_global, weston_view_from_global_fixed,
    weston_view_activate,
    weston_layer_entry
};
use wayland_sys::common::wl_fixed_t;
use wayland_sys::server::wl_signal;
use ::WestonObject;
use ::surface::Surface;
use ::output::Output;
use ::seat::Seat;

pub struct View {
    ptr: *mut weston_view,
    temp: bool,
}

weston_object!(View << weston_view);

impl View {
    pub fn new(surface: &Surface) -> View {
        View::from_ptr(unsafe { weston_view_create(surface.ptr()) })
    }

    pub fn get_position(&self) -> (f32, f32) {
        unsafe { ((*self.ptr).geometry.x, (*self.ptr).geometry.y) }
    }

    pub fn set_position(&self, x: f32, y: f32) {
        unsafe { weston_view_set_position(self.ptr, x, y); }
    }

    pub fn set_transform_parent(&self, parent: &View) {
        unsafe { weston_view_set_transform_parent(self.ptr, parent.ptr()); }
    }

    pub fn set_mask(&self, x: libc::c_int, y: libc::c_int, width: libc::c_int, height: libc::c_int) {
        unsafe { weston_view_set_mask(self.ptr, x, y, width, height); }
    }

    pub fn set_mask_infinite(&self) {
        unsafe { weston_view_set_mask_infinite(self.ptr); }
    }

    pub fn is_mapped(&self) -> bool {
        unsafe { weston_view_is_mapped(self.ptr) }
    }

    pub fn schedule_repaint(&self) {
        unsafe { weston_view_schedule_repaint(self.ptr); }
    }

    pub fn damage_below(&self) {
        unsafe { weston_view_damage_below(self.ptr); }
    }

    // TODO weston_view_move_to_plane

    pub fn unmap(&self) {
        unsafe { weston_view_unmap(self.ptr); }
    }

    pub fn update_transform(&self) {
        unsafe { weston_view_update_transform(self.ptr); }
    }

    pub fn to_global_fixed(&self, sx: wl_fixed_t, sy: wl_fixed_t) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_view_to_global_fixed(self.ptr, sx, sy, &mut x, &mut y); }
        (x, y)
    }

    pub fn to_global_float(&self, sx: f32, sy: f32) -> (f32, f32) {
        let mut x = 0.0;
        let mut y = 0.0;
        unsafe { weston_view_to_global_float(self.ptr, sx, sy, &mut x, &mut y); }
        (x, y)
    }

    pub fn from_global_float(&self, x: f32, y: f32) -> (f32, f32) {
        let mut vx = 0.0;
        let mut vy = 0.0;
        unsafe { weston_view_from_global_float(self.ptr, x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn from_global(&self, x: i32, y: i32) -> (i32, i32) {
        let mut vx = 0;
        let mut vy = 0;
        unsafe { weston_view_from_global(self.ptr, x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn from_global_fixed(&self, x: wl_fixed_t, y: wl_fixed_t) -> (wl_fixed_t, wl_fixed_t) {
        let mut vx = 0;
        let mut vy = 0;
        unsafe { weston_view_from_global_fixed(self.ptr, x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn activate(&self, seat: &Seat, flags: u32) {
        unsafe { weston_view_activate(self.ptr, seat.ptr(), flags); }
    }

    obj_accessors!(View | parent_view = |&this| { (*this.ptr).parent_view });
    obj_accessors!(Surface | surface = |&this| { (*this.ptr).surface });
    obj_accessors!(Output | output = |&this| { (*this.ptr).output });
    prop_accessors!(ptr weston_layer_entry | layer_link);
    prop_accessors!(ptr wl_signal | destroy_signal);
}

impl Drop for View {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_view_destroy(self.ptr); }
        }
    }
}
