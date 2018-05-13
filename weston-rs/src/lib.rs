pub extern crate libweston_sys;
pub extern crate wayland_sys;
pub extern crate wayland_server;
pub extern crate input_sys;
pub extern crate xkbcommon;
pub extern crate libc;
pub extern crate vsprintf;
#[macro_use]
extern crate foreign_types;
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

pub use foreign_types::{ForeignType, ForeignTypeRef};
pub use wayland_sys::common::{
    wl_fixed_from_int, wl_fixed_to_int, wl_fixed_to_double, wl_fixed_from_double
};
pub use wayland_server::{Display, EventLoop, create_display};
pub use wayland_server::protocol::wl_shell_surface::{Resize, Transient};
pub use xkbcommon::xkb;

// These don't need any wrapping, they're just bundles of i32 fields
pub type Geometry = libweston_sys::weston_geometry;
pub type Position = libweston_sys::weston_position;
pub type Size = libweston_sys::weston_size;

macro_rules! prop_accessors {
    ($typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> $typ {
            unsafe { (*self.as_ptr()).$prop }
        })+
    };
    (ptr $typ:ty | $($prop:ident),+) => {
        $(#[inline] pub fn $prop(&self) -> &mut $typ {
            unsafe { &mut (*self.as_ptr()).$prop }
        })+
    }
}

macro_rules! obj_accessors {
    ($typ:ident | $($prop:ident $prop_mut:ident = |&$self:ident| $acc:block),+) => {
        $(
            #[inline] pub fn $prop<'a>(&'a self) -> &$typ {
                use foreign_types::ForeignTypeRef;
                unsafe { $typ::from_ptr({ let $self = &self; $acc }) }
            }

            #[inline] pub fn $prop_mut<'a>(&'a self) -> &mut $typ {
                use foreign_types::ForeignTypeRef;
                unsafe { $typ::from_ptr_mut({ let $self = &self; $acc }) }
            }
        )+
    };
    ($typ:ident<$typp:tt> | $($prop:ident $prop_mut:ident = |&$self:ident| $acc:block),+) => {
        $(
            #[inline] pub fn $prop<'a, $typp>(&'a self) -> &$typ<$typp> {
                use foreign_types::ForeignTypeRef;
                unsafe { $typ::from_ptr({ let $self = &self; $acc }) }
            }

            #[inline] pub fn $prop_mut<'a, $typp>(&'a self) -> &mut $typ<$typp> {
                use foreign_types::ForeignTypeRef;
                unsafe { $typ::from_ptr_mut({ let $self = &self; $acc }) }
            }
        )+
    };
    (opt $typ:ident | $($prop:ident $prop_mut:ident = |&$self:ident| $acc:block),+) => {
        $(
            #[inline] pub fn $prop<'a>(&'a self) -> Option<&$typ> {
                use foreign_types::ForeignTypeRef;
                let ptr = unsafe { let $self = &self; $acc };
                if ptr.is_null() {
                    None
                } else {
                    Some(unsafe { $typ::from_ptr({ let $self = &self; $acc }) })
                }
            }

            #[inline] pub fn $prop_mut<'a>(&'a self) -> Option<&mut $typ> {
                use foreign_types::ForeignTypeRef;
                let ptr = unsafe { let $self = &self; $acc };
                if ptr.is_null() {
                    None
                } else {
                    Some(unsafe { $typ::from_ptr_mut({ let $self = &self; $acc }) })
                }
            }
        )+
    };
    (opt $typ:ident | $($prop:ident = |&$self:ident| $acc:block),+) => {
        $(
            #[inline] pub fn $prop<'a>(&'a self) -> Option<$typ> {
                use foreign_types::ForeignType;
                let ptr = unsafe { let $self = &self; $acc };
                if ptr.is_null() {
                    None
                } else {
                    Some(unsafe { $typ::from_ptr({ let $self = &self; $acc }) })
                }
            }
        )+
    }

}

#[macro_export]
macro_rules! wl_container_of {
    ($ptr:expr, $type:ty, $member:ident) => {{
        ($ptr as *mut u8).offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }}
}

pub mod ev;
pub mod matrix;
pub mod listener;
pub mod compositor;
pub mod launcher;
pub mod launcher_loginw;
pub mod backend;
pub mod output_api;
pub mod output;
pub mod head;
pub mod seat;
pub mod pointer;
pub mod keyboard;
pub mod touch;
pub mod layer;
pub mod surface;
pub mod view;
pub mod desktop;

pub use memoffset::*;
pub use matrix::*;
pub use listener::*;
pub use compositor::*;
pub use launcher::*;
pub use launcher_loginw::*;
pub use backend::*;
pub use output_api::*;
pub use output::*;
pub use head::*;
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
