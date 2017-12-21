use libc;
use std::mem;
use num_traits::FromPrimitive;
use libweston_sys::{
    weston_desktop_api, weston_desktop_surface, weston_seat, weston_output
};
use ::WestonObject;
use ::output::Output;
use ::seat::Seat;
use super::surface::{DesktopSurface, SurfaceEdge};

pub trait DesktopApi<SC> {
    fn surface_added(&mut self, surface: DesktopSurface<SC>);

    fn surface_removed(&mut self, surface: DesktopSurface<SC>);

    fn committed(&mut self, _surface: DesktopSurface<SC>, _sx: i32, _sy: i32) {}

    fn show_window_menu(&mut self, _surface: DesktopSurface<SC>, _seat: Seat, _x: i32, _y: i32) {}

    fn set_parent(&mut self, _surface: DesktopSurface<SC>, _parent: DesktopSurface<SC>) {}

    /// Named like that because `move` is a Rust keyword
    fn moove(&mut self, _surface: DesktopSurface<SC>, _seat: Seat, _serial: u32) {}

    fn resize(&mut self, _surface: DesktopSurface<SC>, _seat: Seat, _serial: u32, _edges: SurfaceEdge) {}

    fn fullscreen_requested(&mut self, _surface: DesktopSurface<SC>, _fullscreen: bool, _output: Output) {}

    fn maximized_requested(&mut self, _surface: DesktopSurface<SC>, _maximized: bool) {}

    fn minimized_requested(&mut self, _surface: DesktopSurface<SC>) {}
}

pub extern "C" fn run_surface_added<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.surface_added(surface);
}

pub extern "C" fn run_surface_removed<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.surface_removed(surface);
}

pub extern "C" fn run_committed<SC>(surface: *mut weston_desktop_surface, sx: i32, sy: i32, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.committed(surface, sx, sy);
}

pub extern "C" fn run_show_window_menu<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, x: i32, y: i32, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let seat = Seat::from_ptr_temporary(seat);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.show_window_menu(surface, seat, x, y);
}

pub extern "C" fn run_set_parent<SC>(surface: *mut weston_desktop_surface, parent: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let parent = DesktopSurface::from_ptr_temporary(parent);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.set_parent(surface, parent);
}

pub extern "C" fn run_move<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, serial: u32, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let seat = Seat::from_ptr_temporary(seat);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.moove(surface, seat, serial);
}

pub extern "C" fn run_resize<SC>(surface: *mut weston_desktop_surface, seat: *mut weston_seat, serial: u32, edges: u32, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let seat = Seat::from_ptr_temporary(seat);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.resize(surface, seat, serial, SurfaceEdge::from_u32(edges).unwrap_or(SurfaceEdge::None));
}

pub extern "C" fn run_fullscreen_requested<SC>(surface: *mut weston_desktop_surface, fullscreen: bool, output: *mut weston_output, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let output = Output::from_ptr_temporary(output);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.fullscreen_requested(surface, fullscreen, output);
}

pub extern "C" fn run_maximized_requested<SC>(surface: *mut weston_desktop_surface, maximized: bool, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.maximized_requested(surface, maximized);
}

pub extern "C" fn run_minimized_requested<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let surface = DesktopSurface::from_ptr_temporary(surface);
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.minimized_requested(surface);
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
    wapi
}
