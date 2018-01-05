pub extern crate libweston_sys;
pub extern crate wayland_sys;
pub extern crate wayland_server;
pub extern crate input_sys;
pub extern crate libc;
pub extern crate vsprintf;
#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;
#[macro_use]
extern crate const_cstr;
#[macro_use]
extern crate memoffset;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_builder;
extern crate loginw;

use std::borrow::Borrow;
use std::os::raw::c_void;

pub trait WestonObject where Self: Sized {
    type T;

    fn from_ptr(ptr: *mut Self::T) -> Self;
    fn from_ptr_temporary(ptr: *mut Self::T) -> Self;
    fn ptr(&self) -> *mut Self::T;

    fn from_void_ptr(ptr: *mut c_void) -> Self {
        Self::from_ptr(ptr as *mut Self::T)
    }

    fn from_void_ptr_temporary(ptr: *mut c_void) -> Self {
        Self::from_ptr_temporary(ptr as *mut Self::T)
    }

    fn same_as<U>(&self, other: U) -> bool where U: Sized + Borrow<Self> {
        self.ptr() == other.borrow().ptr()
    }
}

macro_rules! weston_object {
    ($wrap:ident << $typ:ident $($k:ident : $v:expr),*) => {
        impl ::WestonObject for $wrap {
            type T = $typ;

            #[inline] fn from_ptr(ptr: *mut $typ) -> $wrap {
                $wrap {
                    ptr,
                    temp: false,
                    $($k: $v)*
                }
            }

            #[inline] fn from_ptr_temporary(ptr: *mut $typ) -> $wrap {
                $wrap {
                    ptr,
                    temp: true,
                    $($k: $v)*
                }
            }

            #[inline] fn ptr(&self) -> *mut $typ {
                self.ptr
            }
        }
    };
    ($wrap:ident<$tvar:ident> << $typ:ident $($k:ident : $v:expr),*) => {
        impl<$tvar> ::WestonObject for $wrap<$tvar> {
            type T = $typ;

            #[inline] fn from_ptr(ptr: *mut $typ) -> $wrap<$tvar> {
                $wrap {
                    ptr,
                    temp: false,
                    phantom: ::std::marker::PhantomData::<$tvar>,
                    $($k: $v)*
                }
            }

            #[inline] fn from_ptr_temporary(ptr: *mut $typ) -> $wrap<$tvar> {
                $wrap {
                    ptr,
                    temp: true,
                    phantom: ::std::marker::PhantomData::<$tvar>,
                    $($k: $v)*
                }
            }

            #[inline] fn ptr(&self) -> *mut $typ {
                self.ptr
            }
        }
    };
}

macro_rules! prop_accessors {
    ($typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> $typ {
            unsafe { (*self.ptr).$prop }
        })+
    };
    (ptr $typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> &mut $typ {
            unsafe { &mut (*self.ptr).$prop }
        })+
    }
}

macro_rules! obj_accessors {
    ($typ:ident | $($prop:ident = |&$self:ident| $acc:block),+) => {
        $(#[inline] pub fn $prop<'a>(&'a self) -> $typ {
            use ::WestonObject;
            $typ::from_ptr_temporary(unsafe { let $self = &self; $acc })
        })+
    };
    (opt $typ:ident | $($prop:ident = |&$self:ident| $acc:block),+) => {
        $(#[inline] pub fn $prop<'a>(&'a self) -> Option<$typ> {
            use ::WestonObject;
            let ptr = unsafe { let $self = &self; $acc };
            if ptr.is_null() {
                None
            } else {
                Some($typ::from_ptr_temporary(ptr))
            }
        })+
    }
}

#[macro_export]
macro_rules! wl_container_of {
    ($ptr:expr, $type:ty, $member:ident) => {{
        ($ptr as *mut u8).offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }}
}

pub mod listener;
pub mod display;
pub mod compositor;
pub mod launcher;
pub mod launcher_loginw;
pub mod backend;
pub mod output_api;
pub mod output;
pub mod seat;
pub mod pointer;
pub mod keyboard;
pub mod touch;
pub mod layer;
pub mod surface;
pub mod view;
pub mod desktop;

pub use memoffset::*;
pub use listener::*;
pub use display::*;
pub use compositor::*;
pub use launcher::*;
pub use launcher_loginw::*;
pub use backend::*;
pub use output_api::*;
pub use output::*;
pub use seat::*;
pub use pointer::*;
pub use keyboard::*;
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
