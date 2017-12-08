use libweston_sys::{weston_desktop_client};

pub struct DesktopClient {
    ptr: *mut weston_desktop_client,
}

impl From<*mut weston_desktop_client> for DesktopClient {
    fn from(ptr: *mut weston_desktop_client) -> DesktopClient {
        DesktopClient {
            ptr: ptr,
        }
    }
}

impl DesktopClient {
    pub fn ptr(&self) -> *mut weston_desktop_client {
        self.ptr
    }
}
