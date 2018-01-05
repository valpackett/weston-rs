use libc;

pub mod wayland;
pub mod drm;

pub trait Backend {
    fn id(&self) -> libc::c_int;
}

pub use self::wayland::*;
pub use self::drm::*;
