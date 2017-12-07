// Thanks: https://github.com/sardemff7/not-a-wm/blob/master/main.c

extern crate libc;
extern crate libweston_sys;
#[macro_use]
extern crate weston_rs;
extern crate wayland_sys;

use std::{mem, env, ffi, process};
use libweston_sys::{
    weston_desktop_api
};
use weston_rs::*;
use wayland_sys::server::*;

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

struct Context {
    compositor: Compositor,
    output: WindowedOutput,
    output_pending_listener: wl_listener,
    windows_layer: Box<Layer>,
}

struct SurfaceContext {
    view: Option<View>,
}

weston_callback!{wl unsafe fn output_pending_virtual(
        ctx: &mut Context | output_pending_listener, output: &mut Output) {
    output.set_scale(1);
    output.set_transform(0);
    ctx.output.output_set_size(output, 1280, 720);
    output.enable();
}}

weston_callback!{api(weston_desktop_surface) unsafe fn surface_added(
        dsurf: DesktopSurface<SurfaceContext>, ctx: &mut Context) {
    let mut view = dsurf.create_view();
    ctx.windows_layer.entry_insert(&mut view);
    view.set_position(0.0, -1.0);
    dsurf.get_surface().damage();
    ctx.compositor.schedule_repaint();
    let mut sctx = mem::ManuallyDrop::new(SurfaceContext {
        view: Some(view),
    });
    dsurf.set_user_data(&mut sctx);
}}

weston_callback!{api(weston_desktop_surface) unsafe fn surface_removed(
        dsurf: DesktopSurface<SurfaceContext>, ctx: &mut Context) {
    {
        let mut sctx = dsurf.get_user_data();
        //let mut view = sctx.view.take().unwrap();
        //dsurf.unlink_view(&mut view);
    }
    dsurf.unset_user_data();
}}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    let display = Display::new();
    let mut compositor = Compositor::new(&display);
    compositor.set_xkb_rule_names(None);
    let backend = WaylandBackend::new(&compositor);
    let output = WindowedOutput::new(&compositor);
    let mut output_pending_listener: wl_listener = unsafe { mem::zeroed() };
    output.output_create(&compositor, "weston-rs simple example");
    output_pending_listener.notify = output_pending_virtual;
    unsafe { signal::wl_signal_add(compositor.output_pending_signal(), &mut output_pending_listener); }
    compositor.pending_output_coldplug();

    let mut bg_layer = Layer::new(&compositor);
    bg_layer.set_position(LayerPosition::Background);
    let mut bg_surf = Surface::new(&compositor);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.entry_insert(&mut bg_view);

    let mut desktop_api: weston_desktop_api = unsafe { mem::zeroed() };
    desktop_api.struct_size = mem::size_of::<weston_desktop_api>();
    desktop_api.surface_added = Some(surface_added);
    desktop_api.surface_removed = Some(surface_removed);

    let mut windows_layer = Layer::new(&compositor);
    windows_layer.set_position(LayerPosition::Normal);

    let ctx = Context {
       compositor, output, output_pending_listener, windows_layer
    };

    let desktop = Desktop::new(&ctx.compositor, &desktop_api, &ctx);

    let sock_name = display.add_socket_auto();
    env::remove_var("DISPLAY");
    eprintln!("{:?}", sock_name);
    unsafe { libc::setenv(ffi::CString::new("WAYLAND_DISPLAY").expect("CString").as_ptr(), sock_name.as_ptr(), 1); }

    let _ = process::Command::new("gtk3-demo").spawn().expect("spawn");

    ctx.compositor.wake();
    display.run();
}
