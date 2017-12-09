// Thanks: https://github.com/sardemff7/not-a-wm/blob/master/main.c

extern crate libc;
extern crate libweston_sys;
#[macro_use]
extern crate weston_rs;
extern crate wayland_sys;

use std::{mem, env, ffi, process};
use weston_rs::*;
use wayland_sys::server::*;

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

struct Context<'a> {
    backend: WaylandBackend<'a>,
    output: WindowedOutput<'a>,
    output_pending_listener: wl_listener,
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

struct DesktopImpl<'a> {
    compositor: *mut Compositor,
    windows_layer: Layer<'a>,
}

impl<'a> DesktopApi<SurfaceContext> for DesktopImpl<'a> {
    fn surface_added(&mut self, surface: &mut DesktopSurface<SurfaceContext>) {
        let mut view = surface.create_view();
        self.windows_layer.entry_insert(&mut view);
        view.set_position(0.0, -1.0);
        surface.get_surface().damage();
        unsafe { (*self.compositor).schedule_repaint(); }
        let mut sctx = mem::ManuallyDrop::new(SurfaceContext {
            view: Some(view),
        });
        surface.set_user_data(&mut sctx);
    }

    fn surface_removed(&mut self, surface: &mut DesktopSurface<SurfaceContext>) {
        {
            let _ = surface.get_user_data();
            //let mut view = sctx.view.take().unwrap();
            //surface.unlink_view(&mut view);
        }
        surface.unset_user_data();
    }
}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    let display = Display::new();
    let mut compositor = Compositor::new(&display);
    let compositor_ptr = { &mut compositor as *mut _ };
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

    let mut windows_layer = Layer::new(unsafe { &*compositor_ptr });
    windows_layer.set_position(LayerPosition::Normal);

    // is found as the parent struct in the listener
    let _ = Context {
       backend, output, output_pending_listener
    };

    let desktop_impl = Box::new(DesktopImpl {
        compositor: compositor_ptr, windows_layer
    });

    let desktop = Desktop::new(unsafe { &*compositor_ptr }, desktop_impl);

    let sock_name = display.add_socket_auto();
    env::remove_var("DISPLAY");
    eprintln!("{:?}", sock_name);
    unsafe { libc::setenv(ffi::CString::new("WAYLAND_DISPLAY").expect("CString").as_ptr(), sock_name.as_ptr(), 1); }

    let _ = process::Command::new("gtk3-demo").spawn().expect("spawn");

    unsafe { (*compositor_ptr).wake(); }
    display.run();
}
