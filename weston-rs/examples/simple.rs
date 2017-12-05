// Thanks: https://github.com/sardemff7/not-a-wm/blob/master/main.c

extern crate libc;
extern crate libweston_sys;
#[macro_use]
extern crate weston_rs;
extern crate wayland_sys;

use std::mem;
//use libweston_sys::{ };
use weston_rs::{
    Display, Compositor, WaylandBackend, WindowedOutput, Output, Layer, LayerPosition, Surface, View
};
use wayland_sys::server::*;

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

struct Context {
    output: WindowedOutput,
    output_pending_listener: wl_listener,
}

signal_listener!{unsafe fn output_pending_virtual(ctx: &mut Context | output_pending_listener, output: &mut Output) {
    output.set_scale(1);
    output.set_transform(0);
    ctx.output.output_set_size(output, 1280, 720);
    output.enable();
}}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    let display = Display::new();
    let mut compositor = Compositor::new(&display);
    compositor.set_xkb_rule_names(None);
    let backend = WaylandBackend::new(&compositor);
    let mut ctx = Context {
        output: WindowedOutput::new(&compositor),
        output_pending_listener: unsafe { mem::zeroed() },
    };
    ctx.output.output_create(&compositor, "weston-rs simple example");
    ctx.output_pending_listener.notify = output_pending_virtual;
    unsafe { signal::wl_signal_add(compositor.output_pending_signal(), &mut ctx.output_pending_listener); }
    compositor.pending_output_coldplug();

    let mut bg_layer = Layer::new(&compositor);
    bg_layer.set_position(LayerPosition::Background);
    let mut bg_surf = Surface::new(&compositor);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.entry_insert(&mut bg_view);

    compositor.wake();
    display.run();
}
