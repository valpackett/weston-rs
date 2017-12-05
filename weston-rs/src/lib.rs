extern crate libweston_sys;
extern crate wayland_sys;
extern crate libc;
#[macro_use]
extern crate const_cstr;

pub mod compositor;
pub mod backend;
pub mod output_api;

pub use compositor::Compositor;
pub use backend::{Backend, WaylandBackend};
pub use output_api::WindowedOutput;
