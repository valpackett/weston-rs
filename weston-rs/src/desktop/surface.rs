use libc;
use std::{ptr, ffi, marker};
use libweston_sys::{
    weston_desktop_surface,
    weston_desktop_surface_get_user_data, weston_desktop_surface_set_user_data,
    weston_desktop_surface_get_client, weston_desktop_surface_get_surface,
    weston_desktop_surface_get_title, weston_desktop_surface_get_app_id,
    weston_desktop_surface_get_pid, weston_desktop_surface_get_activated,
    weston_desktop_surface_get_maximized, weston_desktop_surface_get_fullscreen,
    weston_desktop_surface_get_resizing, weston_desktop_surface_get_geometry,
    weston_desktop_surface_get_max_size, weston_desktop_surface_get_min_size,
    weston_desktop_surface_create_view, weston_desktop_surface_unlink_view,
    weston_desktop_surface_propagate_layer, weston_desktop_surface_set_activated,
    weston_desktop_surface_set_fullscreen, weston_desktop_surface_set_maximized,
    weston_desktop_surface_set_resizing, weston_desktop_surface_set_size,
    weston_desktop_surface_close,
    weston_surface_is_desktop_surface, weston_surface_get_desktop_surface,
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
use ::{WestonObject, Geometry, Size};
use ::surface::Surface;
use ::view::View;
use super::client::DesktopClient;

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
    obj_accessors!(DesktopClient | get_client = |&this| { weston_desktop_surface_get_client(this.ptr) });
    obj_accessors!(Surface | get_surface = |&this| { weston_desktop_surface_get_surface(this.ptr) });

    pub fn from_surface(surface: &Surface) -> Option<DesktopSurface<T>> {
        if unsafe { weston_surface_is_desktop_surface(surface.ptr()) } {
            return Some(DesktopSurface::from_ptr_temporary(unsafe { weston_surface_get_desktop_surface(surface.ptr()) }))
        }
        None
    }

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

    pub fn create_view(&self) -> View {
        View::from_ptr(unsafe { weston_desktop_surface_create_view(self.ptr) })
    }

    pub fn unlink_view(&self, view: &mut View) {
        unsafe { weston_desktop_surface_unlink_view(view.ptr()); }
    }

    pub fn propagate_layer(&self) {
        unsafe { weston_desktop_surface_propagate_layer(self.ptr); }
    }

    pub fn set_activated(&self, activated: bool) {
        unsafe { weston_desktop_surface_set_activated(self.ptr, activated); }
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        unsafe { weston_desktop_surface_set_fullscreen(self.ptr, fullscreen); }
    }

    pub fn set_maximized(&self, maximized: bool) {
        unsafe { weston_desktop_surface_set_maximized(self.ptr, maximized); }
    }

    pub fn set_resizing(&self, resizing: bool) {
        unsafe { weston_desktop_surface_set_resizing(self.ptr, resizing); }
    }

    pub fn set_size(&self, width: i32, height: i32) {
        unsafe { weston_desktop_surface_set_size(self.ptr, width, height); }
    }

    pub fn close(&self) {
        unsafe { weston_desktop_surface_close(self.ptr); }
    }

    pub fn get_title(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(weston_desktop_surface_get_title(self.ptr)) }
    }

    pub fn get_app_id(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(weston_desktop_surface_get_app_id(self.ptr)) }
    }

    pub fn get_pid(&self) -> libc::pid_t {
        unsafe { weston_desktop_surface_get_pid(self.ptr) }
    }

    pub fn get_activated(&self) -> bool {
        unsafe { weston_desktop_surface_get_activated(self.ptr) }
    }

    pub fn get_maximized(&self) -> bool {
        unsafe { weston_desktop_surface_get_maximized(self.ptr) }
    }

    pub fn get_fullscreen(&self) -> bool {
        unsafe { weston_desktop_surface_get_fullscreen(self.ptr) }
    }

    pub fn get_resizing(&self) -> bool {
        unsafe { weston_desktop_surface_get_resizing(self.ptr) }
    }

    pub fn get_geometry(&self) -> Geometry {
        unsafe { weston_desktop_surface_get_geometry(self.ptr) }
    }

    pub fn get_max_size(&self) -> Size {
        unsafe { weston_desktop_surface_get_max_size(self.ptr) }
    }

    pub fn get_min_size(&self) -> Size {
        unsafe { weston_desktop_surface_get_min_size(self.ptr) }
    }
}
