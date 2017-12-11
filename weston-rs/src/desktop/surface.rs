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

    pub fn set_user_data(&self, data: Box<T>) -> Option<Box<T>> {
        let prev = self.get_user_data();
        unsafe { weston_desktop_surface_set_user_data(self.ptr, Box::into_raw(data) as *mut libc::c_void); }
        prev
    }

    pub fn get_user_data(&self) -> Option<Box<T>> {
        unsafe {
            let ptr = weston_desktop_surface_get_user_data(self.ptr) as *mut T;
            if ptr.is_null() {
                return None
            }
            let bx = Box::from_raw(ptr);
            weston_desktop_surface_set_user_data(self.ptr, ptr::null_mut());
            Some(bx)
        }
    }

    pub fn borrow_user_data(&self) -> Option<&mut T> {
        unsafe {
            let ptr = weston_desktop_surface_get_user_data(self.ptr) as *mut T;
            if ptr.is_null() {
                return None
            }
            Some(&mut *(ptr))
        }
    }

    pub fn get_surface<'a>(&'a self) -> &'a mut Surface {
        let mut surf = mem::ManuallyDrop::new(unsafe { weston_desktop_surface_get_surface(self.ptr) }.into());
        unsafe {
            mem::transmute::<&mut Surface, &'a mut Surface>(&mut *surf)
        }
    }

    pub fn create_view(&self) -> View {
        unsafe { weston_desktop_surface_create_view(self.ptr).into() }
    }

    pub fn unlink_view(&self, view: &mut View) {
        unsafe { weston_desktop_surface_unlink_view(view.ptr()); }
    }
}
