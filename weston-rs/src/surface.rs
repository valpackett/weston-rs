use libweston_sys::{
    weston_surface, weston_surface_create, weston_surface_destroy,
    weston_surface_set_size, weston_surface_set_color, weston_surface_damage,
    weston_surface_get_main_surface,
};
use ::WestonObject;
use ::compositor::Compositor;

pub struct Surface {
    ptr: *mut weston_surface,
    temp: bool,
}

weston_object!(Surface << weston_surface);

impl Surface {
    pub fn new(compositor: &Compositor) -> Surface {
        Surface::from_ptr(unsafe { weston_surface_create(compositor.ptr()) })
    }

    pub fn set_size(&self, width: i32, height: i32) {
        unsafe { weston_surface_set_size(self.ptr, width, height); }
    }

    pub fn set_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { weston_surface_set_color(self.ptr, red, green, blue, alpha); }
    }

    obj_accessors!(Surface | get_main_surface = |&this| { weston_surface_get_main_surface(this.ptr) });

    pub fn damage(&self) {
        unsafe { weston_surface_damage(self.ptr); }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_surface_destroy(self.ptr); }
        }
    }
}
