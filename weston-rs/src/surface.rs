use libc;
use std::ffi;
use libweston_sys::{
    weston_surface, weston_surface_create, weston_surface_destroy,
    weston_surface_set_size, weston_surface_set_color,
    weston_surface_get_role, weston_surface_set_role,
    weston_surface_damage, weston_surface_schedule_repaint,
    weston_surface_is_mapped, weston_surface_unmap,
    weston_surface_to_buffer_float, weston_surface_get_content_size,
    weston_surface_get_main_surface,
};
use wayland_sys::server::wl_signal;
use wayland_server::Resource;
use wayland_server::commons::Interface;
use foreign_types::{ForeignType, ForeignTypeRef};
use ::compositor::CompositorRef;
use ::output::OutputRef;

foreign_type! {
    type CType = weston_surface;
    fn drop = weston_surface_destroy;
    pub struct Surface;
    pub struct SurfaceRef;
}

impl Surface {
    pub fn new(compositor: &CompositorRef) -> Surface {
        unsafe { Surface::from_ptr(weston_surface_create(compositor.as_ptr())) }
    }
}

impl SurfaceRef {
    obj_accessors!(SurfaceRef | main_surface main_surface_mut = |&this| { weston_surface_get_main_surface(this.as_ptr()) });
    obj_accessors!(opt OutputRef | output output_mut = |&this| { (*this.as_ptr()).output });
    obj_accessors!(CompositorRef | compositor compositor_mut = |&this| { (*this.as_ptr()).compositor });
    //obj_accessors!(Resource | resource resource_mut = |&this| { (*this.as_ptr()).resource });
    prop_accessors!(u32 | output_mask);
    prop_accessors!(i32 | width, height);
    prop_accessors!(ptr wl_signal | destroy_signal, commit_signal);

    pub fn set_size(&mut self, width: i32, height: i32) {
        unsafe { weston_surface_set_size(self.as_ptr(), width, height); }
    }

    pub fn set_color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { weston_surface_set_color(self.as_ptr(), red, green, blue, alpha); }
    }

    pub fn get_role(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(weston_surface_get_role(self.as_ptr())) }
    }

    pub fn set_role<S: AsRef<ffi::CStr>, I: Interface>(&mut self, role_name: S, error_resource: Resource<I>, error_code: u32) -> bool {
        unsafe { weston_surface_set_role(self.as_ptr(), role_name.as_ref().as_ptr(), error_resource.c_ptr(), error_code) == 0 }
    }

    pub fn to_buffer_float(&self, x: f32, y: f32) -> (f32, f32) {
        let mut bx = 0.0;
        let mut by = 0.0;
        unsafe { weston_surface_to_buffer_float(self.as_ptr(), x, y, &mut bx, &mut by); }
        (bx, by)
    }

    pub fn is_mapped(&self) -> bool {
        unsafe { weston_surface_is_mapped(self.as_ptr()) }
    }

    pub fn schedule_repaint(&mut self) {
        unsafe { weston_surface_schedule_repaint(self.as_ptr()); }
    }

    pub fn damage(&mut self) {
        unsafe { weston_surface_damage(self.as_ptr()); }
    }

    pub fn unmap(&mut self) {
        unsafe { weston_surface_unmap(self.as_ptr()); }
    }

    pub fn get_content_size(&self) -> (libc::c_int, libc::c_int) {
        let mut width = 0;
        let mut height = 0;
        unsafe { weston_surface_get_content_size(self.as_ptr(), &mut width, &mut height); }
        (width, height)
    }

    pub fn set_committed<D, T: FnMut(&mut SurfaceRef, i32, i32, &mut D) + 'static>(&mut self, cb: T, user_data: D) {
        unsafe {
            (*self.as_ptr()).committed_private = Box::into_raw(Box::new((cb, user_data))) as *mut libc::c_void;
            (*self.as_ptr()).committed = Some(run_callback::<D, T>);
        }
    }

    pub unsafe fn committed_private_mut<D>(&self) -> &mut D {
        &mut *((*self.as_ptr()).committed_private as *mut D)
    }
}

#[allow(unused_unsafe)]
extern "C" fn run_callback<D, T: FnMut(&mut SurfaceRef, i32, i32, &mut D)>(surface: *mut weston_surface, sx: i32, sy: i32) {
    let (mut cb, mut user_data) = unsafe { *Box::from_raw((*surface).committed_private as *mut (T, D)) };
    cb(unsafe { SurfaceRef::from_ptr_mut(surface) }, sx, sy, &mut user_data);
    unsafe { (*surface).committed_private = Box::into_raw(Box::new((cb, user_data))) as *mut libc::c_void };
}
