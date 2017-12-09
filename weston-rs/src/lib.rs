pub extern crate libweston_sys;
pub extern crate wayland_sys;
pub extern crate libc;
pub extern crate vsprintf;
#[macro_use]
extern crate const_cstr;
//#[macro_use]
extern crate memoffset;

macro_rules! prop_accessors {
    ($typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> &mut $typ {
            unsafe { &mut (*self.ptr).$prop }
        })+
    }
}

pub mod display;
pub mod compositor;
pub mod backend;
pub mod output_api;
pub mod output;
pub mod layer;
pub mod surface;
pub mod view;
pub mod desktop;

pub use memoffset::*;
pub use display::*;
pub use compositor::*;
pub use backend::*;
pub use output_api::*;
pub use output::*;
pub use layer::*;
pub use surface::*;
pub use view::*;
pub use desktop::*;

#[macro_export]
macro_rules! wl_container_of {
    ($ptr:expr, $type:ident, $member:ident) => {{
        ($ptr as *mut u8).offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }}
}

#[macro_export]
macro_rules! weston_callback {
    (wl unsafe fn $name:ident ($ctxarg:tt : &mut $ctxtyp:tt | $field:ident, $datarg:tt : &mut $dattyp:ty) $b:block) => {
        #[allow(unused_unsafe)]
        unsafe extern "C" fn $name(listener: *mut ::wayland_sys::server::wl_listener, data: *mut ::std::os::raw::c_void) {
            let mut __data: $dattyp = data.into();
            {
                let $ctxarg = &mut *wl_container_of!(listener, $ctxtyp, $field);
                let $datarg = &mut __data;
                $b;
            }
            ::std::mem::forget(__data);
        }
    };
}


#[macro_export]
macro_rules! weston_logger {
    (fn $name:ident ($strarg:ident : &str) $b:block) => {
        unsafe extern "C" fn $name(fmt: *const ::libc::c_char,
                                   args: *mut ::weston_rs::libweston_sys::__va_list_tag) -> ::libc::c_int {
            let $strarg = ::weston_rs::vsprintf::vsprintf(fmt, args).expect("vsprintf");
            $b;
            0
        }
    }
}

pub fn log_set_handler(
    logger: unsafe extern "C" fn(*const libc::c_char, *mut libweston_sys::__va_list_tag) -> libc::c_int,
    logger_cont: unsafe extern "C" fn(*const libc::c_char, *mut libweston_sys::__va_list_tag) -> libc::c_int,
    ) {
    unsafe {
        libweston_sys::weston_log_set_handler(Some(logger), Some(logger_cont));
    }
}
