use libc;
use std::mem;
use libweston_sys::{
    weston_desktop_api, weston_desktop_surface
};
use ::output::Output;
use super::surface::DesktopSurface;

pub trait DesktopApi<SC> {
    fn surface_added(&mut self, surface: &mut DesktopSurface<SC>);

    fn surface_removed(&mut self, surface: &mut DesktopSurface<SC>);

    fn committed(&mut self, _surface: &mut DesktopSurface<SC>, _sx: i32, _sy: i32) {}

    // fn show_window_menu(&mut self, _surface: &mut DesktopSurface<SC>, _seat: Seat, _x: i32, _y: i32) {}

    fn set_parent(&mut self, _surface: &mut DesktopSurface<SC>, _parent: &mut DesktopSurface<SC>) {}

    // fn move(&mut self, _surface: &mut DesktopSurface<SC>, _seat: &mut Seat, _serial: u32) {}

    // fn resize(&mut self, _surface: &mut DesktopSurface<SC>, _seat: &mut Seat, _serial: u32, _edges: SurfaceEdge) {}

    fn fullscreen_requested(&mut self, _surface: &mut DesktopSurface<SC>, _fullscreen: bool, _output: &mut Output) {}

    fn maximized_requested(&mut self, _surface: &mut DesktopSurface<SC>, _maximized: bool) {}

    fn minimized_requested(&mut self, _surface: &mut DesktopSurface<SC>) {}
}

pub extern "C" fn run_surface_added<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let mut surface = mem::ManuallyDrop::new(surface.into());
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.surface_added(&mut surface);
}

pub extern "C" fn run_surface_removed<SC>(surface: *mut weston_desktop_surface, user_data: *mut libc::c_void) {
    let mut surface = mem::ManuallyDrop::new(surface.into());
    let api = unsafe { &mut *(user_data as *mut Box<DesktopApi<SC>>) };
    api.surface_removed(&mut surface);
}

pub fn make_weston_api<SC>() -> Box<weston_desktop_api> {
    let mut wapi: Box<weston_desktop_api> = Box::new(unsafe { mem::zeroed() });
    (*wapi).struct_size = mem::size_of::<weston_desktop_api>();
    (*wapi).surface_added = Some(run_surface_added::<SC>);
    (*wapi).surface_removed = Some(run_surface_removed::<SC>);
    wapi
}
