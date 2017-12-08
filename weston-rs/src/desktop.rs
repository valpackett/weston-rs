use libc;
use std::{mem, ptr, marker};
use libweston_sys::{
    weston_desktop, weston_desktop_create, weston_desktop_destroy,
    weston_desktop_api,
    weston_desktop_client, weston_desktop_surface,
    weston_desktop_surface_get_user_data, weston_desktop_surface_set_user_data,
    weston_desktop_surface_get_surface,
    weston_desktop_surface_create_view, weston_desktop_surface_unlink_view
};
use super::compositor::Compositor;
use super::surface::Surface;
use super::view::View;


pub struct DesktopClient {
    ptr: *mut weston_desktop_client,
}

impl From<*mut weston_desktop_client> for DesktopClient {
    fn from(ptr: *mut weston_desktop_client) -> DesktopClient {
        DesktopClient {
            ptr: ptr,
        }
    }
}

impl DesktopClient {
    pub fn ptr(&self) -> *mut weston_desktop_client {
        self.ptr
    }
}


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


pub struct Desktop<'comp, UD: 'comp> {
    ptr: *mut weston_desktop,
    phantom: marker::PhantomData<(&'comp Compositor, &'comp UD)>,
}

impl<'comp, UD> Desktop<'comp, UD> {
    pub fn new(compositor: &'comp Compositor, api: &'comp weston_desktop_api, user_data: &'comp UD) -> Desktop<'comp, UD> {
        Desktop {
            ptr: unsafe { weston_desktop_create(compositor.ptr(), api, user_data as *const UD as *mut _) },
            phantom: marker::PhantomData,
        }
    }

    pub fn ptr(&self) -> *mut weston_desktop {
        self.ptr
    }
}

impl<'comp, UD> Drop for Desktop<'comp, UD> {
    fn drop(&mut self) {
        unsafe { weston_desktop_destroy(self.ptr); }
    }
}
