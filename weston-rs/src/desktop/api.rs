use libc;
use std::{mem, any};
use libweston_sys::{
    weston_desktop_api, weston_desktop_surface, weston_desktop_client,
    weston_seat, weston_output,
};
use wayland_server::protocol::wl_shell_surface::Resize;
use foreign_types::ForeignTypeRef;
use ::output::OutputRef;
use ::seat::SeatRef;
use super::surface::DesktopSurfaceRef;
use super::client::DesktopClientRef;

pub trait DesktopApi<SC> {
    fn as_any(&mut self) -> &mut any::Any;

    fn ping_timeout(&mut self, _client: &mut DesktopClientRef) {}

    fn pong(&mut self, _client: &mut DesktopClientRef) {}

    fn surface_added(&mut self, _surface: &mut DesktopSurfaceRef<SC>);

    fn surface_removed(&mut self, _surface: &mut DesktopSurfaceRef<SC>);

    fn committed(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _sx: i32, _sy: i32) {}

    fn show_window_menu(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _seat: &mut SeatRef, _x: i32, _y: i32) {}

    fn set_parent(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _parent: &mut DesktopSurfaceRef<SC>) {}

    /// Named like that because `move` is a Rust keyword
    fn moove(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _seat: &mut SeatRef, _serial: u32) {}

    fn resize(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _seat: &mut SeatRef, _serial: u32, _edges: Resize) {}

    fn fullscreen_requested(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _fullscreen: bool, _output: &mut OutputRef) {}

    fn maximized_requested(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _maximized: bool) {}

    fn minimized_requested(&mut self, _surface: &mut DesktopSurfaceRef<SC>) {}

    /// Position suggestion for an Xwayland window
    ///
    /// X11 applications assume they can position their windows as necessary,
    /// which is not possible in Wayland where positioning is driven by the
    /// shell alone. This function is used to relay absolute position wishes
    /// from Xwayland clients to the shell.
    /// 
    /// This is particularly used for mapping windows at specified locations,
    /// e.g. via the commonly used '-geometry' command line option. In such
    /// case, a call to surface_added() is immediately followed by
    /// xwayland_position() if the X11 application specified a position.
    /// The committed() call that will map the window occurs later, so it
    /// is recommended to usually store and honour the given position for
    /// windows that are not yet mapped.
    /// 
    /// Calls to this function may happen also at other times.
    ///
    /// The given coordinates are in the X11 window system coordinate frame
    /// relative to the X11 root window. Care should be taken to ensure the
    /// window gets mapped to coordinates that correspond to the proposed
    /// position from the X11 client perspective.
    fn set_xwayland_position(&mut self, _surface: &mut DesktopSurfaceRef<SC>, _x: i32, _y: i32) {}
}

pub unsafe extern "C" fn run_ping_timeout<SC>(client: *mut weston_desktop_client, user_data: *mut libc::c_void) {
    let client = DesktopClientRef::from_ptr_mut(client);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.ping_timeout(client);
}

pub unsafe extern "C" fn run_pong<SC>(client: *mut weston_desktop_client, user_data: *mut libc::c_void) {
    let client = DesktopClientRef::from_ptr_mut(client);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.pong(client);
}

pub unsafe extern "C" fn run_surface_added<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.surface_added(surface);
}

pub unsafe extern "C" fn run_surface_removed<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.surface_removed(surface);
}

pub unsafe extern "C" fn run_committed<SC>(surface: *mut weston_desktop_surface, sx: i32, sy: i32, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.committed(surface, sx, sy);
}

pub unsafe extern "C" fn run_show_window_menu<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, x: i32, y: i32, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let seat = SeatRef::from_ptr_mut(seat);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.show_window_menu(surface, seat, x, y);
}

pub unsafe extern "C" fn run_set_parent<SC>(surface: *mut weston_desktop_surface, parent: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let parent = DesktopSurfaceRef::from_ptr_mut(parent);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.set_parent(surface, parent);
}

pub unsafe extern "C" fn run_move<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, serial: u32, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let seat = SeatRef::from_ptr_mut(seat);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.moove(surface, seat, serial);
}

pub unsafe extern "C" fn run_resize<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, serial: u32, edges: u32, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let seat = SeatRef::from_ptr_mut(seat);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.resize(surface, seat, serial, Resize::from_raw(edges).unwrap_or(Resize::None));
}

pub unsafe extern "C" fn run_fullscreen_requested<SC>(surface: *mut weston_desktop_surface, fullscreen: bool, output: *mut weston_output, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let output = OutputRef::from_ptr_mut(output);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.fullscreen_requested(surface, fullscreen, output);
}

pub unsafe extern "C" fn run_maximized_requested<SC>(surface: *mut weston_desktop_surface, maximized: bool, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.maximized_requested(surface, maximized);
}

pub unsafe extern "C" fn run_minimized_requested<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.minimized_requested(surface);
}

pub unsafe extern "C" fn run_set_xwayland_position<SC>(surface: *mut weston_desktop_surface, x: i32, y: i32, user_data: *mut libc::c_void) {
    let surface = DesktopSurfaceRef::from_ptr_mut(surface);
    let api = &mut *(user_data as *mut Box<DesktopApi<SC>>);
    api.set_xwayland_position(surface, x, y);
}

pub fn make_weston_api<SC>() -> Box<weston_desktop_api> {
    let mut wapi: Box<weston_desktop_api> = Box::new(unsafe { mem::zeroed() });
    (*wapi).struct_size = mem::size_of::<weston_desktop_api>();
    (*wapi).surface_added = Some(run_surface_added::<SC>);
    (*wapi).surface_removed = Some(run_surface_removed::<SC>);
    (*wapi).committed = Some(run_committed::<SC>);
    (*wapi).show_window_menu = Some(run_show_window_menu::<SC>);
    (*wapi).set_parent = Some(run_set_parent::<SC>);
    (*wapi).move_ = Some(run_move::<SC>);
    (*wapi).resize = Some(run_resize::<SC>);
    (*wapi).fullscreen_requested = Some(run_fullscreen_requested::<SC>);
    (*wapi).maximized_requested = Some(run_maximized_requested::<SC>);
    (*wapi).minimized_requested = Some(run_minimized_requested::<SC>);
    (*wapi).set_xwayland_position = Some(run_set_xwayland_position::<SC>);
    wapi
}
