// Thanks: https://github.com/sardemff7/not-a-wm/blob/master/main.c

extern crate libc;
extern crate vsprintf;
extern crate libweston_sys;
extern crate weston_rs;
extern crate wayland_sys;
#[macro_use]
extern crate memoffset;

use std::{mem, os};
use libweston_sys::{
    weston_log_set_handler, __va_list_tag,
    weston_output, weston_output_set_scale, weston_output_set_transform, weston_output_enable
};
use weston_rs::{Compositor, WaylandBackend, WindowedOutput};
use wayland_sys::server::*;

unsafe extern "C" fn wlog(fmt: *const libc::c_char, args: *mut __va_list_tag) -> i32 {
    eprintln!("WESTON: {}", vsprintf::vsprintf(fmt, args).expect("vsprintf"));
    0
}

// XXX: does not actually work with members other than the first one
macro_rules! wl_container_of {
    ($ptr:expr, $type:ident, $member:ident) => {
        $ptr.offset(-1 * offset_of!($type, $member) as isize) as *mut $type
    }
}

#[repr(C)]
struct Context {
    output_pending_listener: wl_listener,
    test: usize,
    output: WindowedOutput,
}

unsafe extern "C" fn output_pending_virtual(listener: *mut wl_listener, data: *mut os::raw::c_void) {
    eprintln!("output_pending_virtual offs {}", offset_of!(Context, test));
    eprintln!("output_pending_virtual offs {}", offset_of!(Context, output_pending_listener));
    let ctx = wl_container_of!(listener, Context, output_pending_listener);
    let woutput = data as *mut weston_output;
    weston_output_set_scale(woutput, 1);
    weston_output_set_transform(woutput, 0);
    (*ctx).output.output_set_size(woutput, 1280, 720);
    weston_output_enable(woutput);
}

fn main() {
    unsafe {
        weston_log_set_handler(Some(wlog), Some(wlog));
    }
    let display = unsafe { wl_display_create() };
    let mut compositor = Compositor::new(display);
    compositor.set_xkb_rule_names(None);
    let backend = WaylandBackend::new(&compositor);
    let mut ctx = Context {
        test: 123,
        output: WindowedOutput::new(&compositor),
        output_pending_listener: unsafe { mem::zeroed() },
    };
    ctx.output.output_create(&compositor, "my window");
    ctx.output_pending_listener.notify = output_pending_virtual;
    unsafe { signal::wl_signal_add(&mut (*compositor.ptr()).output_pending_signal, &mut ctx.output_pending_listener); }
    compositor.pending_output_coldplug();
    compositor.wake();
    unsafe { wl_display_run(display); }
}
