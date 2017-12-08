use libweston_sys::{
    weston_view, weston_view_create, weston_view_destroy,
    weston_view_set_position,
    weston_layer_entry
};
use ::surface::Surface;

pub struct View {
    ptr: *mut weston_view,
}

impl From<*mut weston_view> for View {
    fn from(ptr: *mut weston_view) -> View {
        View {
            ptr,
        }
    }
}

impl View {
    pub fn new(surface: &Surface) -> View {
        View {
            ptr: unsafe { weston_view_create(surface.ptr()) }
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        unsafe { weston_view_set_position(self.ptr, x, y); }
    }

    prop_accessors!(weston_layer_entry | layer_link);

    pub fn ptr(&self) -> *mut weston_view {
        self.ptr
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe { weston_view_destroy(self.ptr); }
    }
}
