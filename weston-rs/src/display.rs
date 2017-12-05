use wayland_sys::server::*;

pub struct Display {
    ptr: *mut wl_display,
}

impl Display {
    pub fn new() -> Display {
        Display {
            ptr: unsafe { wl_display_create() },
        }
    }

    pub fn run(&self) {
        unsafe { wl_display_run(self.ptr); }
    }

    pub fn ptr(&self) -> *mut wl_display {
        self.ptr
    }
}
