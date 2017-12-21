use libweston_sys::{
    weston_view, weston_view_create, weston_view_destroy,
    weston_view_set_position,
    weston_layer_entry
};
use ::WestonObject;
use ::surface::Surface;

pub struct View {
    ptr: *mut weston_view,
    temp: bool,
}

weston_object!(View << weston_view);

impl View {
    pub fn new(surface: &Surface) -> View {
        View::from_ptr(unsafe { weston_view_create(surface.ptr()) })
    }

    pub fn set_position(&self, x: f32, y: f32) {
        unsafe { weston_view_set_position(self.ptr, x, y); }
    }

    prop_accessors!(ptr weston_layer_entry | layer_link);

    obj_accessors!(Surface | surface = |&this| { (*this.ptr).surface });
}

impl Drop for View {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_view_destroy(self.ptr); }
        }
    }
}
