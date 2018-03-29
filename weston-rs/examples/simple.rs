//! This is a very simple compositor, like
//! <https://github.com/sardemff7/not-a-wm/blob/master/main.c>
//! but in Rust and with a little bit more stuff (e.g. window movement)

extern crate libc;
extern crate loginw;
#[macro_use]
extern crate weston_rs;
#[macro_use]
extern crate lazy_static;

use std::{env, ffi, process};
use weston_rs::*;
use loginw::priority;

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
    focus_count: i16,
}

/// User data for the Desktop API implementation
struct DesktopImpl<'a> {
    windows_layer: Layer<'a>,
    stack: Vec<DesktopSurface<SurfaceContext>>,
}

impl<'a> DesktopApi<SurfaceContext> for DesktopImpl<'a> {
    fn surface_added(&mut self, dsurf: DesktopSurface<SurfaceContext>) {
        let mut view = dsurf.create_view();
        self.windows_layer.entry_insert(&mut view);
        view.set_position(0.0, -1.0);
        dsurf.get_surface().damage();
        COMPOSITOR.schedule_repaint();
        if let Some(focus) = self.stack.last() {
            //focus.set_activated(false);
        }
        self.stack.push(dsurf.temp_clone());
        //dsurf.set_activated(true);
        // NOTE: activate causes SIGBUS in wl_signal_emit when there's no keyboard???
        view.activate(&COMPOSITOR.first_seat().expect("first_seat"), ActivateFlag::CONFIGURE);
        let _ = dsurf.set_user_data(Box::new(SurfaceContext {
            view,
            focus_count: 0,
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
                        dx: f64::from(view_x) - wl_fixed_to_double(pointer.grab_x()),
                        dy: f64::from(view_y) - wl_fixed_to_double(pointer.grab_y()),
                    };
                    pointer.start_grab(grab);
                }
            }
        }
    }
}

fn activate(focus_view: View, seat: Seat, flags: ActivateFlag) {
    let main_surf = focus_view.surface().get_main_surface();
    if let Some(dsurf) = DesktopSurface::<SurfaceContext>::from_surface(&main_surf) {
        focus_view.activate(&seat, flags);
    }
}

fn click_activate(p: Pointer) {
    if !p.is_default_grab() {
        return;
    }
    if let Some(focus_view) = p.focus() {
        activate(focus_view, p.seat(), ActivateFlag::CONFIGURE | ActivateFlag::CLICKED);
    }
}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);
    COMPOSITOR.set_xkb_rule_names(None); // defaults to environment variables

    // Backend setup
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

    // Background color
    let mut bg_layer = Layer::new(&*COMPOSITOR);
    bg_layer.set_position(POSITION_BACKGROUND);
    let bg_surf = Surface::new(&*COMPOSITOR);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.entry_insert(&mut bg_view);

    // Layer for user applications
    let mut windows_layer = Layer::new(&*COMPOSITOR);
    windows_layer.set_position(POSITION_NORMAL);

    // Our data for libweston-desktop stuff
    let mut desktop_impl = Box::new(DesktopImpl {
        windows_layer,
        stack: Vec::new(),
    });
    let desktop_impl_ptr = &mut *desktop_impl as *mut DesktopImpl; // TODO figure out safe way

    // The libweston-desktop object
    // NOTE: Important to keep around (do not do 'let _')
    let _desktop = Desktop::new(&*COMPOSITOR, desktop_impl);

    // Left click to focus window
    let _ = COMPOSITOR.add_button_binding(0x110, KeyboardModifier::empty(), &|p, _, _| click_activate(p));
    // Right click to focus window
    let _ = COMPOSITOR.add_button_binding(0x111, KeyboardModifier::empty(), &|p, _, _| click_activate(p));
    // XXX: popup windows are not handled correctly
    WlListener::new(Box::new(move |p: Keyboard| {
        println!("FOCUS KEYBOARD");
        let mut desktop_impl = unsafe { &mut (*desktop_impl_ptr) };
        if let Some(dsurf) = desktop_impl.stack.last() {
            dsurf.set_activated(false);
        }
        if let Some(focus) = p.focus() {
            //focus.activate(&p.seat(), 0);
            if let Some(dsurf) = DesktopSurface::<SurfaceContext>::from_surface(&focus) {
                if let Some(pos) = desktop_impl.stack.iter().position(|s| s.same_as(&dsurf)) {
                    let _ = desktop_impl.stack.remove(pos);
                }
                desktop_impl.stack.push(dsurf.temp_clone());
                dsurf.set_activated(true);
            }
        }
    })).signal_add(COMPOSITOR.first_seat().expect("first_seat").get_keyboard().expect("first_seat get_keyboard").focus_signal());

    // Ctrl+Enter to spawn a terminal
    COMPOSITOR.add_key_binding(28, KeyboardModifier::CTRL, &|_, _, _| {
        use std::os::unix::process::CommandExt;
        let _ = process::Command::new("weston-terminal").before_exec(|| {
            // loginw sets realtime priority for the compositor
            // see https://blog.martin-graesslin.com/blog/2017/09/kwinwayland-goes-real-time/ for reasons
            // we obviously don't want it in user applications :D
            priority::make_normal();
            Ok(())
        }).spawn().expect("spawn");
    });

    // Set environment for spawned processes (namely, the terminal above)
    env::remove_var("DISPLAY");
    let sock_name = DISPLAY.add_socket_auto();
    unsafe { libc::setenv(ffi::CString::new("WAYLAND_DISPLAY").expect("CString").as_ptr(), sock_name.as_ptr(), 1); }

    // Go!
    COMPOSITOR.wake();
    DISPLAY.run();
}
