use libweston_sys::{
    weston_surface, weston_surface_create, weston_surface_destroy,
    weston_surface_set_size, weston_surface_set_color
};
use super::compositor::Compositor;

pub struct Surface {
    ptr: *mut weston_surface,
}

impl Surface {
    pub fn new(compositor: &Compositor) -> Surface {
        Surface {
            ptr: unsafe { weston_surface_create(compositor.ptr()) }
        }
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        unsafe { weston_surface_set_size(self.ptr, width, height); }
    }

    pub fn set_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { weston_surface_set_color(self.ptr, red, green, blue, alpha); }
    }

    pub fn ptr(&self) -> *mut weston_surface {
        self.ptr
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { weston_surface_destroy(self.ptr); }
    }
}
