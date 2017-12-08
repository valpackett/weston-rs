use libc;
use std::{ptr, mem, marker};
use libweston_sys::{
    weston_desktop_surface,
    weston_desktop_surface_get_user_data, weston_desktop_surface_set_user_data,
    weston_desktop_surface_get_surface,
    weston_desktop_surface_create_view, weston_desktop_surface_unlink_view
};
use ::surface::Surface;
use ::view::View;

pub struct DesktopSurface<T> {
    ptr: *mut weston_desktop_surface,
    phantom: marker::PhantomData<T>,
}

impl<T> From<*mut weston_desktop_surface> for DesktopSurface<T> {
    fn from(ptr: *mut weston_desktop_surface) -> DesktopSurface<T> {
        DesktopSurface {
            ptr: ptr,
            phantom: marker::PhantomData::<T>,
        }
    }
}

impl<T> DesktopSurface<T> {
    pub fn ptr(&self) -> *mut weston_desktop_surface {
        self.ptr
    }

    pub fn set_user_data(&mut self, data: &mut T) {
        unsafe { weston_desktop_surface_set_user_data(self.ptr, data as *mut _ as *mut libc::c_void); }
    }

    pub fn get_user_data(&self) -> &mut T {
        unsafe { &mut *(weston_desktop_surface_get_user_data(self.ptr) as *mut T) }
    }

    pub fn unset_user_data(&mut self) {
        unsafe { weston_desktop_surface_set_user_data(self.ptr, ptr::null_mut()); }
    }

    pub fn get_surface<'a>(&'a self) -> &'a mut Surface {
        let mut surf = mem::ManuallyDrop::new(unsafe { weston_desktop_surface_get_surface(self.ptr) }.into());
        unsafe {
            mem::transmute::<&mut Surface, &'a mut Surface>(&mut *surf)
        }
    }

    pub fn create_view(&mut self) -> View {
        unsafe { weston_desktop_surface_create_view(self.ptr).into() }
    }

    pub fn unlink_view(&self, view: &mut View) {
        unsafe { weston_desktop_surface_unlink_view(view.ptr()); }
    }
}
