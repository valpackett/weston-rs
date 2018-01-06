//! This is a very simple compositor, like
//! https://github.com/sardemff7/not-a-wm/blob/master/main.c
//! but in Rust and with a little bit more stuff (e.g. window movement)

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

/// Mouse handler for moving windows
struct MoveGrab {
    dsurf: DesktopSurface<SurfaceContext>,
    dx: f64,
    dy: f64,
}

impl PointerGrab for MoveGrab {
    fn motion(&mut self, pointer: &mut Pointer, _time: &libc::timespec, event: PointerMotionEvent) {
        pointer.moove(event);
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        sctx.view.set_position((wl_fixed_to_double(pointer.x()) + self.dx) as f32, (wl_fixed_to_double(pointer.y()) + self.dy) as f32);
        self.dsurf.get_surface().compositor().schedule_repaint();
    }

    fn button(&mut self, pointer: &mut Pointer, _time: &libc::timespec, _button: u32, state: ButtonState) {
        if pointer.button_count() == 0 && state == ButtonState::Released {
            pointer.end_grab();
        }
    }

    fn cancel(&mut self, pointer: &mut Pointer) {
        pointer.end_grab();
    }
}


/// Per-surface user data for Desktop Surfaces (libweston-desktop's wrapper around surfaces)
struct SurfaceContext {
    view: View,
}

/// User data for the Desktop API implementation
struct DesktopImpl<'a> {
    windows_layer: Layer<'a>,
}

impl<'a> DesktopApi<SurfaceContext> for DesktopImpl<'a> {
    fn surface_added(&mut self, dsurf: DesktopSurface<SurfaceContext>) {
        let mut view = dsurf.create_view();
        self.windows_layer.entry_insert(&mut view);
        view.set_position(0.0, -1.0);
        dsurf.get_surface().damage();
        COMPOSITOR.schedule_repaint();
        let _ = dsurf.set_user_data(Box::new(SurfaceContext {
            view,
        }));
    }

    fn surface_removed(&mut self, dsurf: DesktopSurface<SurfaceContext>) {
        let mut sctx = dsurf.get_user_data().expect("user_data");
        dsurf.unlink_view(&mut sctx.view);
        // sctx dropped here, destroying the view
    }

    fn moove(&mut self, dsurf: DesktopSurface<SurfaceContext>, seat: Seat, serial: u32) {
        let sctx = dsurf.borrow_user_data().expect("user_data");
        if let Some(pointer) = seat.get_pointer() {
            if let Some(focus) = pointer.focus() {
                if pointer.button_count() > 0 && serial == pointer.grab_serial() &&
                    focus.surface().get_main_surface().same_as(dsurf.get_surface()) {
                    let (view_x, view_y) = sctx.view.get_position();
                    let grab = MoveGrab {
                        dsurf: dsurf.temp_clone(),
                        dx: view_x as f64 - wl_fixed_to_double(pointer.grab_x()),
                        dy: view_y as f64 - wl_fixed_to_double(pointer.grab_y()),
                    };
                    pointer.start_grab(grab);
                }
            }
        }
    }
}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    COMPOSITOR.set_xkb_rule_names(None); // defaults to environment variables
    if env::var("LOGINW_FD").is_ok() {
        let launcher = LoginwLauncher::connect(&*COMPOSITOR, 0, &std::ffi::CString::new("default").unwrap(), false).expect("connect");
        COMPOSITOR.set_launcher(launcher);
        let _backend = DrmBackend::new(&*COMPOSITOR, DrmBackendConfigBuilder::default().build().unwrap());
        let output_api = COMPOSITOR.get_drm_output().expect("get_drm_output");
        WlListener::new(Box::new(move |ou: Output| {
            output_api.set_mode(&ou, DrmBackendOutputMode::Current, None);
            ou.set_scale(1);
            ou.set_extra_scale(1.0);
            ou.set_transform(0);
            output_api.set_gbm_format(&ou, None);
            ou.enable();
        })).signal_add(COMPOSITOR.output_pending_signal());
    } else {
        let _backend = WaylandBackend::new(&*COMPOSITOR, WaylandBackendConfigBuilder::default().build().unwrap());
        let output_api = COMPOSITOR.get_windowed_output().expect("get_windowed_output");
        output_api.output_create(&*COMPOSITOR, "weston-rs simple example");
        WlListener::new(Box::new(move |ou: Output| {
            ou.set_scale(1);
            ou.set_extra_scale(1.0);
            ou.set_transform(0);
            output_api.output_set_size(&ou, 1280, 720);
            ou.enable();
        })).signal_add(COMPOSITOR.output_pending_signal());
    }
    COMPOSITOR.pending_output_coldplug();

    let mut bg_layer = Layer::new(&*COMPOSITOR);
    bg_layer.set_position(POSITION_BACKGROUND);
    let bg_surf = Surface::new(&*COMPOSITOR);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.entry_insert(&mut bg_view);

    let mut windows_layer = Layer::new(&*COMPOSITOR);
    windows_layer.set_position(POSITION_NORMAL);

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
