use libc;
use libweston_sys::{
    weston_activate_flag_WESTON_ACTIVATE_FLAG_NONE,
    weston_activate_flag_WESTON_ACTIVATE_FLAG_CONFIGURE,
    weston_activate_flag_WESTON_ACTIVATE_FLAG_CLICKED,
    weston_view, weston_view_create, weston_view_destroy,
    weston_view_set_position, weston_view_set_transform_parent,
    weston_view_set_mask, weston_view_set_mask_infinite,
    weston_view_is_mapped, weston_view_schedule_repaint,
    weston_view_damage_below, weston_view_unmap,
    weston_view_update_transform, weston_view_geometry_dirty,
    weston_view_to_global_fixed, weston_view_to_global_float,
    weston_view_from_global_float, weston_view_from_global, weston_view_from_global_fixed,
    weston_view_activate,
    weston_layer_entry, weston_layer_entry_remove
};
use wayland_sys::common::wl_fixed_t;
use wayland_sys::server::wl_signal;
use foreign_types::{ForeignType, ForeignTypeRef};
use ::surface::SurfaceRef;
use ::output::OutputRef;
use ::seat::SeatRef;

bitflags! {
    #[derive(Default)]
    pub struct ActivateFlag: u32 {
        const NONE = weston_activate_flag_WESTON_ACTIVATE_FLAG_NONE;
        const CONFIGURE = weston_activate_flag_WESTON_ACTIVATE_FLAG_CONFIGURE;
        const CLICKED = weston_activate_flag_WESTON_ACTIVATE_FLAG_CLICKED;
    }
}

foreign_type! {
    type CType = weston_view;
    fn drop = weston_view_destroy;
    pub struct View;
    pub struct ViewRef;
}

impl View {
    pub fn new(surface: &SurfaceRef) -> View {
        unsafe { View::from_ptr(weston_view_create(surface.as_ptr())) }
    }
}

impl ViewRef {
    obj_accessors!(opt ViewRef | parent_view parent_view_mut = |&this| { (*this.as_ptr()).parent_view });
    obj_accessors!(SurfaceRef | surface surface_mut = |&this| { (*this.as_ptr()).surface });
    obj_accessors!(opt OutputRef | output output_mut = |&this| { (*this.as_ptr()).output });
    prop_accessors!(ptr weston_layer_entry | layer_link);
    prop_accessors!(ptr wl_signal | destroy_signal);

    pub fn get_position(&self) -> (f32, f32) {
        unsafe { ((*self.as_ptr()).geometry.x, (*self.as_ptr()).geometry.y) }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        unsafe { weston_view_set_position(self.as_ptr(), x, y); }
    }

    pub fn set_transform_parent(&mut self, parent: &ViewRef) {
        unsafe { weston_view_set_transform_parent(self.as_ptr(), parent.as_ptr()); }
    }

    pub fn set_mask(&mut self, x: libc::c_int, y: libc::c_int, width: libc::c_int, height: libc::c_int) {
        unsafe { weston_view_set_mask(self.as_ptr(), x, y, width, height); }
    }

    pub fn set_mask_infinite(&mut self) {
        unsafe { weston_view_set_mask_infinite(self.as_ptr()); }
    }

    pub fn is_mapped(&self) -> bool {
        unsafe { weston_view_is_mapped(self.as_ptr()) }
    }

    pub fn schedule_repaint(&mut self) {
        unsafe { weston_view_schedule_repaint(self.as_ptr()); }
    }

    pub fn damage_below(&mut self) {
        unsafe { weston_view_damage_below(self.as_ptr()); }
    }

    // TODO weston_view_move_to_plane

    pub fn unmap(&mut self) {
        unsafe { weston_view_unmap(self.as_ptr()); }
    }

    pub fn update_transform(&mut self) {
        unsafe { weston_view_update_transform(self.as_ptr()); }
    }

    pub fn geometry_dirty(&mut self) {
        unsafe { weston_view_geometry_dirty(self.as_ptr()); }
    }

    pub fn layer_entry_remove(&mut self) {
        unsafe { weston_layer_entry_remove(self.layer_link()); }
    }

    pub fn to_global_fixed(&self, sx: wl_fixed_t, sy: wl_fixed_t) -> (wl_fixed_t, wl_fixed_t) {
        let mut x = 0;
        let mut y = 0;
        unsafe { weston_view_to_global_fixed(self.as_ptr(), sx, sy, &mut x, &mut y); }
        (x, y)
    }

    pub fn to_global_float(&self, sx: f32, sy: f32) -> (f32, f32) {
        let mut x = 0.0;
        let mut y = 0.0;
        unsafe { weston_view_to_global_float(self.as_ptr(), sx, sy, &mut x, &mut y); }
        (x, y)
    }

    pub fn from_global_float(&self, x: f32, y: f32) -> (f32, f32) {
        let mut vx = 0.0;
        let mut vy = 0.0;
        unsafe { weston_view_from_global_float(self.as_ptr(), x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn from_global(&self, x: i32, y: i32) -> (i32, i32) {
        let mut vx = 0;
        let mut vy = 0;
        unsafe { weston_view_from_global(self.as_ptr(), x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn from_global_fixed(&self, x: wl_fixed_t, y: wl_fixed_t) -> (wl_fixed_t, wl_fixed_t) {
        let mut vx = 0;
        let mut vy = 0;
        unsafe { weston_view_from_global_fixed(self.as_ptr(), x, y, &mut vx, &mut vy); }
        (vx, vy)
    }

    pub fn activate(&mut self, seat: &SeatRef, flags: ActivateFlag) {
        unsafe { weston_view_activate(self.as_ptr(), seat.as_ptr(), flags.bits()); }
    }
}
