use libc;
use std::{ptr, marker};
use libweston_sys::{
    weston_desktop_surface,
    weston_desktop_surface_get_user_data, weston_desktop_surface_set_user_data,
    weston_desktop_surface_get_surface,
    weston_desktop_surface_create_view, weston_desktop_surface_unlink_view,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_NONE,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_LEFT,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP_LEFT,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM_LEFT,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_RIGHT,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP_RIGHT,
    weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM_RIGHT
};
use ::WestonObject;
use ::surface::Surface;
use ::view::View;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum SurfaceEdge {
    None = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_NONE,
    Top = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP,
    Bottom = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM,
    Left = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_LEFT,
    TopLeft = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP_LEFT,
    BottomLeft = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM_LEFT,
    Right = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_RIGHT,
    TopRight = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_TOP_RIGHT,
    BottomRight = weston_desktop_surface_edge_WESTON_DESKTOP_SURFACE_EDGE_BOTTOM_RIGHT,
}

#[allow(dead_code)]
pub struct DesktopSurface<T> {
    ptr: *mut weston_desktop_surface,
    temp: bool,
    phantom: marker::PhantomData<T>,
}

weston_object!(DesktopSurface<T> << weston_desktop_surface);

impl<T> DesktopSurface<T> {
    pub fn temp_clone(&self) -> DesktopSurface<T> {
        DesktopSurface {
            ptr: self.ptr,
            temp: true,
            phantom: marker::PhantomData::<T>,
        }
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

    obj_accessors!(Surface | get_surface = |&this| { weston_desktop_surface_get_surface(this.ptr) });

    pub fn create_view(&self) -> View {
        View::from_ptr(unsafe { weston_desktop_surface_create_view(self.ptr) })
    }

    pub fn unlink_view(&self, view: &mut View) {
        unsafe { weston_desktop_surface_unlink_view(view.ptr()); }
    }
}
