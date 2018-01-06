use libc;
use libweston_sys::{
    weston_surface, weston_surface_create, weston_surface_destroy,
    weston_surface_set_size, weston_surface_set_color, weston_surface_damage,
    weston_surface_schedule_repaint,
    weston_surface_is_mapped, weston_surface_unmap,
    weston_surface_to_buffer_float, weston_surface_get_content_size,
    weston_surface_get_main_surface,
};
use wayland_sys::server::wl_signal;
use ::WestonObject;
use ::compositor::Compositor;
use ::output::Output;

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

    pub fn to_buffer_float(&self, x: f32, y: f32) -> (f32, f32) {
        let mut bx = 0.0;
        let mut by = 0.0;
        unsafe { weston_surface_to_buffer_float(self.ptr, x, y, &mut bx, &mut by); }
        (bx, by)
    }

    pub fn is_mapped(&self) -> bool {
        unsafe { weston_surface_is_mapped(self.ptr) }
    }

    pub fn schedule_repaint(&self) {
        unsafe { weston_surface_schedule_repaint(self.ptr); }
    }

    pub fn damage(&self) {
        unsafe { weston_surface_damage(self.ptr); }
    }

    pub fn unmap(&self) {
        unsafe { weston_surface_unmap(self.ptr); }
    }

    pub fn get_content_size(&self) -> (libc::c_int, libc::c_int) {
        let mut width = 0;
        let mut height = 0;
        unsafe { weston_surface_get_content_size(self.ptr, &mut width, &mut height); }
        (width, height)
    }

    obj_accessors!(Surface | get_main_surface = |&this| { weston_surface_get_main_surface(this.ptr) });
    obj_accessors!(Output | output = |&this| { (*this.ptr).output });
    obj_accessors!(Compositor | compositor = |&this| { (*this.ptr).compositor });
    prop_accessors!(u32 | output_mask);
    prop_accessors!(ptr wl_signal | destroy_signal, commit_signal);
}

impl Drop for Surface {
    fn drop(&mut self) {
        if !self.temp {
            unsafe { weston_surface_destroy(self.ptr); }
        }
    }
}
