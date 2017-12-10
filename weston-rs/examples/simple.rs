// Thanks: https://github.com/sardemff7/not-a-wm/blob/master/main.c

extern crate libc;
extern crate libweston_sys;
#[macro_use]
extern crate weston_rs;
extern crate wayland_sys;

use std::{mem, env, ffi, process};
use weston_rs::*;

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

struct SurfaceContext {
    view: Option<View>,
}

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
    let _ = WaylandBackend::new(&compositor);
    let output_api = WindowedOutput::new(unsafe { &*compositor_ptr });
    output_api.output_create(&compositor, "weston-rs simple example");
    let output_api_ptr = { &output_api as *const _ };
    WlListener::new(Box::new(move |ou: &mut Output| {
        ou.set_scale(1);
        ou.set_transform(0);
        let oapi: &WindowedOutput = unsafe { &*output_api_ptr };
        oapi.output_set_size(&ou, 1280, 720);
        ou.enable();
    })).signal_add(compositor.output_pending_signal());
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
