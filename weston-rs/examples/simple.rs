//! This is a very simple compositor, like
//! https://github.com/sardemff7/not-a-wm/blob/master/main.c
//! but in Rust.

extern crate libc;
#[macro_use]
extern crate weston_rs;
#[macro_use]
extern crate lazy_static;

use std::{env, ffi, process};
use weston_rs::*;

lazy_static! {
    static ref DISPLAY: Display = Display::new();
    static ref COMPOSITOR: Compositor = Compositor::new(&*DISPLAY);
}

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

/// Per-surface user data for Desktop Surfaces (libweston-desktop's wrapper around surfaces)
struct SurfaceContext {
    view: View,
}

/// User data for the Desktop API implementation
struct DesktopImpl<'a> {
    windows_layer: Layer<'a>,
}

impl<'a> DesktopApi<SurfaceContext> for DesktopImpl<'a> {
    fn surface_added(&mut self, surface: &mut DesktopSurface<SurfaceContext>) {
        let mut view = surface.create_view();
        self.windows_layer.entry_insert(&mut view);
        view.set_position(0.0, -1.0);
        surface.get_surface().damage();
        COMPOSITOR.schedule_repaint();
        let _ = surface.set_user_data(Box::new(SurfaceContext {
            view,
        }));
    }

    fn surface_removed(&mut self, surface: &mut DesktopSurface<SurfaceContext>) {
        let mut sctx = surface.get_user_data().expect("user_data");
        surface.unlink_view(&mut sctx.view);
        // sctx dropped here, destroying the view
    }
}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    COMPOSITOR.set_xkb_rule_names(None); // defaults to environment variables
    let _backend = WaylandBackend::new(&*COMPOSITOR);
    let output_api = WindowedOutput::new(&*COMPOSITOR);
    output_api.output_create(&*COMPOSITOR, "weston-rs simple example");
    WlListener::new(Box::new(move |ou: &mut Output| {
        ou.set_scale(1);
        ou.set_transform(0);
        output_api.output_set_size(&ou, 1280, 720);
        ou.enable();
    })).signal_add(COMPOSITOR.output_pending_signal());
    COMPOSITOR.pending_output_coldplug();

    let mut bg_layer = Layer::new(&*COMPOSITOR);
    bg_layer.set_position(LayerPosition::Background);
    let bg_surf = Surface::new(&*COMPOSITOR);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.entry_insert(&mut bg_view);

    let mut windows_layer = Layer::new(&*COMPOSITOR);
    windows_layer.set_position(LayerPosition::Normal);

    let desktop_impl = Box::new(DesktopImpl {
        windows_layer
    });

    // Important to keep around. `let _ = …` blows up
    let _desktop = Desktop::new(&*COMPOSITOR, desktop_impl);

    env::remove_var("DISPLAY");
    let sock_name = DISPLAY.add_socket_auto();
    unsafe { libc::setenv(ffi::CString::new("WAYLAND_DISPLAY").expect("CString").as_ptr(), sock_name.as_ptr(), 1); }

    let _ = process::Command::new("gtk3-demo").spawn().expect("spawn");

    COMPOSITOR.wake();
    DISPLAY.run();
}
