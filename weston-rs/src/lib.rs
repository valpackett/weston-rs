pub extern crate libweston_sys;
pub extern crate wayland_sys;
pub extern crate libc;
pub extern crate vsprintf;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;
#[macro_use]
extern crate const_cstr;
#[macro_use]
extern crate memoffset;

macro_rules! prop_accessors {
    ($typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> &mut $typ {
            unsafe { &mut (*self.ptr).$prop }
        })+
    }
}

macro_rules! obj_accessors {
    ($typ:ty | $($prop:ident = |&$self:ident| $acc:block),+) => {
        $(#[inline] pub fn $prop<'a>(&'a self) -> &'a mut $typ {
            let mut obj = ::std::mem::ManuallyDrop::new((unsafe { let $self = &self; $acc }).into());
            unsafe { ::std::mem::transmute::<&mut $typ, &'a mut $typ>(&mut *obj) }
        })+
    };
    (opt $typ:ty | $($prop:ident = |&$self:ident| $acc:block),+) => {
        $(#[inline] pub fn $prop<'a>(&'a self) -> Option<&'a mut $typ> {
            let ptr = unsafe { let $self = &self; $acc };
            if ptr.is_null() {
                None
            } else {
                let mut obj = ::std::mem::ManuallyDrop::new(ptr.into());
                Some(unsafe { ::std::mem::transmute::<&mut $typ, &'a mut $typ>(&mut *obj) })
            }
        })+
    }
}

pub mod listener;
pub mod display;
pub mod compositor;
pub mod backend;
pub mod output_api;
pub mod output;
pub mod seat;
pub mod pointer;
pub mod touch;
pub mod layer;
pub mod surface;
pub mod view;
pub mod desktop;

pub use memoffset::*;
pub use listener::*;
pub use display::*;
pub use compositor::*;
pub use backend::*;
pub use output_api::*;
pub use output::*;
pub use seat::*;
pub use pointer::*;
pub use touch::*;
pub use layer::*;
pub use surface::*;
pub use view::*;
pub use desktop::*;

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
