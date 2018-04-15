//! This is a very simple compositor, like
//! <https://github.com/sardemff7/not-a-wm/blob/master/main.c>
//! but in Rust and with a little bit more stuff (e.g. window movement)

#![feature(nll)]

extern crate libc;
extern crate loginw;
#[macro_use]
extern crate weston_rs;
#[macro_use]
extern crate lazy_static;
extern crate mut_static;

use std::{env, ffi, process, cell, any, cmp};
use mut_static::MutStatic;
use weston_rs::*;
use loginw::priority;

lazy_static! {
    static ref COMPOSITOR: MutStatic<Compositor> = MutStatic::new();
    static ref DESKTOP: MutStatic<Desktop<SurfaceContext>> = MutStatic::new();
}

weston_logger!{fn wlog(msg: &str) {
    eprint!("WESTON: {}", msg);
}}

weston_logger!{fn wlog_continue(msg: &str) {
    eprint!("{}", msg);
}}

/// Mouse handler for moving windows
struct MoveGrab<'a> {
    dsurf: &'a mut DesktopSurfaceRef<SurfaceContext>,
    dx: f64,
    dy: f64,
}

impl<'a> PointerGrab for MoveGrab<'a> {
    fn motion(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, event: PointerMotionEvent) {
        pointer.moove(event);
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        sctx.view.set_position((wl_fixed_to_double(pointer.x()) + self.dx) as f32, (wl_fixed_to_double(pointer.y()) + self.dy) as f32);
        self.dsurf.surface().compositor_mut().schedule_repaint();
    }

    fn button(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, _button: u32, state: ButtonState) {
        if pointer.button_count() == 0 && state == ButtonState::Released {
            pointer.end_grab();
        }
    }

    fn cancel(&mut self, pointer: &mut PointerRef) {
        pointer.end_grab();
    }
}

/// Mouse handler for resizing windows
struct ResizeGrab<'a> {
    dsurf: &'a mut DesktopSurfaceRef<SurfaceContext>,
    edges: Resize,
    width: i32,
    height: i32,
}

impl<'a> PointerGrab for ResizeGrab<'a> {
    fn motion(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, event: PointerMotionEvent) {
        pointer.moove(event);
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        let (from_x, from_y) = sctx.view.from_global_fixed(pointer.grab_x(), pointer.grab_y());
        let (to_x, to_y) = sctx.view.from_global_fixed(pointer.x(), pointer.y());
        let mut width = self.width;
        if self.edges.contains(Resize::Left) {
            width += wl_fixed_to_int(from_x - to_x);
        } else if self.edges.contains(Resize::Right) {
            width += wl_fixed_to_int(to_x - from_x);
        }
        let mut height = self.height;
        if self.edges.contains(Resize::Top) {
            height += wl_fixed_to_int(from_y - to_y);
        } else if self.edges.contains(Resize::Bottom) {
            height += wl_fixed_to_int(to_y - from_y);
        }
        let mut min_size = self.dsurf.get_min_size();
        min_size.width = cmp::max(1, min_size.width);
        min_size.height = cmp::max(1, min_size.height);
        let max_size = self.dsurf.get_max_size();
        if width < min_size.width {
            width = min_size.width;
        } else if max_size.width > 0 && width > max_size.width {
            width = max_size.width;
        }
        if height < min_size.height {
            height = min_size.height;
        } else if max_size.width > 0 && width > max_size.width {
            // is it right that we're doing the width thing again, not height? (copied from weston desktop shell)
            width = max_size.width;
        }
        self.dsurf.set_size(width, height);
    }

    fn button(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, _button: u32, state: ButtonState) {
        if pointer.button_count() == 0 && state == ButtonState::Released {
            self.cancel(pointer);
        }
    }

    fn cancel(&mut self, pointer: &mut PointerRef) {
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        self.dsurf.set_resizing(false);
        sctx.resize_edges = Resize::None;
        pointer.end_grab();
    }
}


/// Per-surface user data for Desktop Surfaces (libweston-desktop's wrapper around surfaces)
struct SurfaceContext {
    view: View,
    focus_count: i16,
    /// Commit handling must check if a resize with top/left edges is happening
    /// and move the surface accordingly
    resize_edges: Resize,
    last_width: f32,
    last_height: f32,
}

/// User data for the Desktop API implementation
struct DesktopImpl {
    windows_layer: Layer,
}

impl DesktopApi<SurfaceContext> for DesktopImpl {
    fn as_any(&mut self) -> &mut any::Any { self }

    fn surface_added(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>) {
        let mut view = dsurf.create_view();
        self.windows_layer.view_list_entry_insert(&mut view);
        let mut compositor = COMPOSITOR.write().expect("compositor MutStatic");
        dsurf.surface_mut().damage();
        compositor.schedule_repaint();
        dsurf.set_activated(true);
        view.activate(&compositor.first_seat().expect("first_seat"), ActivateFlag::CONFIGURE);
        let _ = dsurf.set_user_data(Box::new(SurfaceContext {
            view,
            resize_edges: Resize::None,
            last_width: 0.0,
            last_height: 0.0,
            focus_count: 1,
        }));
    }

    fn surface_removed(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>) {
        let mut sctx = dsurf.get_user_data().expect("user_data");
        dsurf.unlink_view(&mut sctx.view);
        // sctx dropped here, destroying the view
    }

    fn committed(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, _sx: i32, _sy: i32) {
        let sctx = dsurf.borrow_user_data().expect("user_data");
        let surface = dsurf.surface();
        let (from_x, from_y) = sctx.view.from_global_float(0.0, 0.0);
        let (to_x, to_y) = sctx.view.from_global_float(
            if sctx.resize_edges.contains(Resize::Left) { sctx.last_width - surface.width() as f32 } else { 0.0 },
            if sctx.resize_edges.contains(Resize::Top) { sctx.last_height - surface.height() as f32 } else { 0.0 },
        );
        let (orig_x, orig_y) = sctx.view.get_position();
        sctx.view.set_position(orig_x + to_x - from_x, orig_y + to_y - from_y);
        sctx.last_width = surface.width() as f32;
        sctx.last_height = surface.height() as f32;
    }

    fn moove(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, seat: &mut SeatRef, serial: u32) {
        let sctx = dsurf.borrow_user_data().expect("user_data");
        if let Some(pointer) = seat.pointer_mut() {
            if let Some(focus) = pointer.focus() {
                if pointer.button_count() > 0 && serial == pointer.grab_serial() &&
                    focus.surface().main_surface().as_ptr() == dsurf.surface().as_ptr() {
                    let (view_x, view_y) = sctx.view.get_position();
                    let grab = MoveGrab {
                        dsurf: unsafe { DesktopSurfaceRef::from_ptr_mut(dsurf.as_ptr()) },
                        dx: f64::from(view_x) - wl_fixed_to_double(pointer.grab_x()),
                        dy: f64::from(view_y) - wl_fixed_to_double(pointer.grab_y()),
                    };
                    pointer.start_grab(grab);
                }
            }
        }
    }

    fn resize(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, seat: &mut SeatRef, serial: u32, edges: Resize) {
        if edges == Resize::None || edges.contains(Resize::Left | Resize::Right) || edges.contains(Resize::Top | Resize::Bottom) {
            return
        }
        let sctx = dsurf.borrow_user_data().expect("user_data");
        if let Some(pointer) = seat.pointer_mut() {
            if let Some(focus) = pointer.focus() {
                if pointer.button_count() > 0 && serial == pointer.grab_serial() &&
                    focus.surface().main_surface().as_ptr() == dsurf.surface().as_ptr() {
                    let geom = dsurf.get_geometry();
                    let grab = ResizeGrab {
                        dsurf: unsafe { DesktopSurfaceRef::from_ptr_mut(dsurf.as_ptr()) },
                        edges,
                        width: geom.width,
                        height: geom.height,
                    };
                    dsurf.set_resizing(true);
                    sctx.resize_edges = edges;
                    pointer.start_grab(grab);
                }
            }
        }
    }
}

fn activate(view: &mut ViewRef, seat: &SeatRef, flags: ActivateFlag) {
    // "cannot borrow *view as mutable" even with nll
    let main_surf = unsafe { SurfaceRef::from_ptr(view.surface().main_surface().as_ptr()) };
    if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&main_surf) {
        let mut desktop = DESKTOP.write().expect("desktop MutStatic");
        let desktop_impl = desktop.api().as_any().downcast_mut::<DesktopImpl>().expect("DesktopImpl downcast");

        view.activate(&seat, flags);

        // Re-insert into the layer to put on top visually
        if view.layer_link().layer.is_null() {
            // Except for newly created surfaces (?)
            // e.g. w/o this, clicking a GTK menu action that spawns a new window would freeze
            return
        }
        view.geometry_dirty();
        view.layer_entry_remove();
        desktop_impl.windows_layer.view_list_entry_insert(view);
        dsurf.propagate_layer();
        view.geometry_dirty();
        dsurf.surface_mut().damage();
    }
}

fn click_activate(p: &mut PointerRef) {
    if !p.is_default_grab() {
        return;
    }
    if let Some(focus_view) = p.focus_mut() {
        activate(focus_view, p.seat(), ActivateFlag::CONFIGURE | ActivateFlag::CLICKED);
    }
}

fn main() {
    weston_rs::log_set_handler(wlog, wlog_continue);

    let (mut display, mut event_loop) = create_display();
    let mut compositor = Compositor::new(&display, &mut event_loop);

    compositor.set_xkb_rule_names(None); // defaults to environment variables

    // Backend setup
    if env::var("LOGINW_FD").is_ok() {
        let launcher = LoginwLauncher::connect(&compositor, &mut event_loop, 0, &std::ffi::CString::new("default").unwrap(), false).expect("connect");
        compositor.set_launcher(launcher);
        let _backend = DrmBackend::new(&compositor, DrmBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { DrmOutputImplRef::from_ptr(compositor.get_drm_output().expect("get_drm_output").as_ptr()) };
        WlListener::new(Box::new(move |ou: &mut OutputRef| {
            output_api.set_mode(&ou, DrmBackendOutputMode::Current, None);
            ou.set_scale(1);
            ou.set_extra_scale(1.0);
            ou.set_transform(0);
            output_api.set_gbm_format(&ou, None);
            ou.enable();
        })).signal_add(compositor.output_pending_signal());
    } else {
        let _backend = WaylandBackend::new(&compositor, WaylandBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { WindowedOutputImplRef::from_ptr(compositor.get_windowed_output().expect("get_windowed_output").as_ptr()) };
        output_api.output_create(&compositor, "weston-rs simple example");
        WlListener::new(Box::new(move |ou: &mut OutputRef| {
            ou.set_scale(1);
            ou.set_extra_scale(1.0);
            ou.set_transform(0);
            output_api.output_set_size(&ou, 1280, 720);
            ou.enable();
        })).signal_add(compositor.output_pending_signal());
    }
    compositor.pending_output_coldplug();

    // Background color
    let mut bg_layer = Layer::new(&compositor);
    bg_layer.set_position(POSITION_BACKGROUND);
    let mut bg_surf = Surface::new(&compositor);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let mut bg_view = View::new(&bg_surf);
    bg_layer.view_list_entry_insert(&mut bg_view);

    // Layer for user applications
    let mut windows_layer = Layer::new(&compositor);
    windows_layer.set_position(POSITION_NORMAL);

    // Our data for libweston-desktop stuff
    let desktop_impl = Box::new(DesktopImpl {
        windows_layer,
    });

    // The libweston-desktop object
    // NOTE: Important to keep around (do not do 'let _')
    let desktop = Desktop::new(unsafe { CompositorRef::from_ptr(compositor.as_ptr()) }, desktop_impl);

    // Left click to focus window
    let _ = compositor.add_button_binding(ev::BTN_LEFT, KeyboardModifier::empty(), &|p, _, _| click_activate(p));
    // Right click to focus window
    let _ = compositor.add_button_binding(ev::BTN_RIGHT, KeyboardModifier::empty(), &|p, _, _| click_activate(p));

    let focused_surface = cell::RefCell::new(None); // in desktop-shell this is part of seat state
    WlListener::new(Box::new(move |p: &mut KeyboardRef| {

        if let Some(old_focus) = focused_surface.replace(p.focus().map(|f| unsafe { SurfaceRef::from_ptr(f.as_ptr()) })) {
            if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&old_focus) {
                if let Some(sctx) = dsurf.borrow_user_data() {
                    sctx.focus_count -= 1;
                    if sctx.focus_count == 0 {
                        dsurf.set_activated(false);
                    }
                }
            }
        }

        if let Some(focus) = *focused_surface.borrow() {
            if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&focus) {
                if let Some(sctx) = dsurf.borrow_user_data() {
                    if sctx.focus_count == 0 {
                        dsurf.set_activated(true);
                    }
                    sctx.focus_count += 1;
                }
            }
        }
    })).signal_add(compositor.first_seat().expect("first_seat").keyboard().expect("first_seat keyboard").focus_signal());

    // Ctrl+Enter to spawn a terminal
    compositor.add_key_binding(ev::KEY_ENTER, KeyboardModifier::CTRL, &|_, _, _| {
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
    let sock_name = display.add_socket_auto().expect("add_socket_auto");
    use std::os::unix::ffi::OsStrExt;
    unsafe { libc::setenv(
            ffi::CString::new("WAYLAND_DISPLAY").expect("CString").as_ptr(),
            sock_name.as_bytes().first().unwrap() as *const u8 as *const _, 1); }

    // Go!
    compositor.wake();
    COMPOSITOR.set(compositor).expect("compositor MutStatic set");
    DESKTOP.set(desktop).expect("desktop MutStatic set");
    event_loop.run();
}
