use std::{ptr, mem};
use libc;
use libweston_sys::{
    weston_compositor, weston_compositor_create, weston_compositor_destroy,
    weston_compositor_shutdown,
    weston_compositor_set_xkb_rule_names,
    weston_compositor_wake, weston_compositor_schedule_repaint,
    weston_compositor_flush_heads_changed, weston_compositor_add_heads_changed_listener,
    weston_seat,
    weston_binding, weston_compositor_add_key_binding,
    weston_compositor_add_modifier_binding, weston_compositor_add_button_binding,
    weston_compositor_add_touch_binding, weston_compositor_add_axis_binding,
    weston_compositor_add_debug_binding, weston_install_debug_key_binding,
    weston_compositor_run_key_binding, weston_compositor_run_modifier_binding,
    weston_compositor_run_button_binding, weston_compositor_run_touch_binding,
    weston_compositor_run_axis_binding, weston_compositor_run_debug_binding,
    weston_compositor_iterate_heads, weston_compositor_create_output_with_head,
    weston_keyboard, weston_keyboard_modifier, weston_pointer, weston_touch,
    weston_pointer_axis_event,
    weston_head,
    weston_launcher, launcher_interface,
};
use xkbcommon::xkb;
use xkbcommon::xkb::ffi::{xkb_rule_names, xkb_context_ref};
use wayland_sys::server::wl_signal;
use foreign_types::{ForeignType, ForeignTypeRef};
use wayland_server::{Display, EventLoop};
use ::layer::LayerRef;
use ::launcher::Launcher;
use ::listener::WlListener;
use ::seat::SeatRef;
use ::pointer::{PointerRef, PointerAxisEvent, Axis};
use ::keyboard::{KeyboardRef, KeyboardModifier};
use ::touch::TouchRef;
use ::output::Output;
use ::head::HeadRef;

/// Opaque reference to a key/modifier/button/touch/axis/debug binding.
/// Hold on to it if you want to later destroy the binding.
pub struct Binding(*mut weston_binding);

extern "C" fn run_key_binding<F: FnMut(&mut KeyboardRef, &libc::timespec, u32)>(keyboard: *mut weston_keyboard, time: *const libc::timespec, key: u32, data: *mut libc::c_void) {
    let cb = unsafe { &mut *(data as *mut F) };
    cb(unsafe { KeyboardRef::from_ptr_mut(keyboard) }, unsafe { &*time }, key);
}

extern "C" fn run_modifier_binding<F: FnMut(&mut KeyboardRef, KeyboardModifier)>(keyboard: *mut weston_keyboard, modifier: weston_keyboard_modifier, data: *mut libc::c_void) {
    let cb = unsafe { &mut *(data as *mut F) };
    cb(unsafe { KeyboardRef::from_ptr_mut(keyboard) }, KeyboardModifier::from_bits_truncate(modifier));
}

extern "C" fn run_button_binding<F: FnMut(&mut PointerRef, &libc::timespec, u32)>(pointer: *mut weston_pointer, time: *const libc::timespec, button: u32, data: *mut libc::c_void) {
    let cb = unsafe { &mut *(data as *mut F) };
    cb(unsafe { PointerRef::from_ptr_mut(pointer) }, unsafe { &*time }, button);
}

extern "C" fn run_touch_binding<F: FnMut(&mut TouchRef, &libc::timespec)>(touch: *mut weston_touch, time: *const libc::timespec, data: *mut libc::c_void) {
    let cb = unsafe { &mut *(data as *mut F) };
    cb(unsafe { TouchRef::from_ptr_mut(touch) }, unsafe { &*time });
}

extern "C" fn run_axis_binding<F: FnMut(&mut PointerRef, &libc::timespec, PointerAxisEvent)>(pointer: *mut weston_pointer, time: *const libc::timespec, event: *mut weston_pointer_axis_event , data: *mut libc::c_void) {
    let cb = unsafe { &mut *(data as *mut F) };
    cb(unsafe { PointerRef::from_ptr_mut(pointer) }, unsafe { &*time }, unsafe { &*event }.into());
}

pub struct HeadIterator<'a> {
    compositor: &'a CompositorRef,
    head: *mut weston_head,
}

impl<'a> Iterator for HeadIterator<'a> {
    type Item = &'a mut HeadRef;

    fn next(&mut self) -> Option<&'a mut HeadRef> {
        self.head = unsafe { weston_compositor_iterate_heads(self.compositor.as_ptr(), self.head) };
        if self.head.is_null() {
            None
        } else {
            Some(unsafe { HeadRef::from_ptr_mut(self.head) })
        }
    }
}

foreign_type! {
    type CType = weston_compositor;
    fn drop = weston_compositor_destroy;
    pub struct Compositor;
    pub struct CompositorRef;
}

unsafe impl Sync for Compositor {}

impl Compositor {
    pub fn new(display: &Display, event_loop: *mut EventLoop) -> Compositor {
        let ptr = unsafe {
            // The event loop is stored as user data. Used in launcher callbacks.
            weston_compositor_create(display.c_ptr(), event_loop as *mut _)
        };
        // TODO check ptr != null
        let mut result = unsafe { Compositor::from_ptr(ptr) };
        unsafe { (*result.as_ptr()).user_data = &mut result as *mut _ as *mut libc::c_void };
        result
    }
}

impl CompositorRef {
    obj_accessors!(LayerRef |
                   fade_layer fade_layer_mut = |&this| { &mut (*this.as_ptr()).fade_layer },
                   cursor_layer cursor_layer_mut = |&this| { &mut (*this.as_ptr()).cursor_layer });
    obj_accessors!(opt SeatRef |
                   first_seat first_seat_mut = |&this| { wl_container_of!((*this.as_ptr()).seat_list.next, weston_seat, link) });
    prop_accessors!(
        ptr wl_signal | destroy_signal, create_surface_signal, activate_signal, transform_signal,
        kill_signal, idle_signal, wake_signal, show_input_panel_signal, hide_input_panel_signal,
        update_input_panel_signal, seat_created_signal, heads_changed_signal, output_created_signal,
        output_destroyed_signal, output_moved_signal, output_resized_signal, session_signal);
    prop_accessors!(i32 | kb_repeat_rate, kb_repeat_delay);

    pub fn set_session_active(&mut self, active: bool) {
        unsafe { (*self.as_ptr()).session_active = active as _; }
    }

    pub fn set_launcher<T: Launcher>(&mut self, launcher: T) {
        unsafe { (*self.as_ptr()).launcher = launcher.into_weston(); }
    }

    pub fn set_launcher_raw(&mut self, wrapper: *mut weston_launcher) {
        unsafe { (*self.as_ptr()).launcher = wrapper; }
    }

    pub fn get_xkb_context(&self) -> xkb::Context {
        unsafe { xkb::Context::from_raw_ptr(xkb_context_ref((*self.as_ptr()).xkb_context)) }
    }

    pub fn set_xkb_rule_names(&mut self, names: Option<*mut xkb_rule_names>) {
        unsafe { weston_compositor_set_xkb_rule_names(self.as_ptr(), names.unwrap_or(ptr::null_mut())); }
    }

    pub fn schedule_repaint(&mut self) {
        unsafe { weston_compositor_schedule_repaint(self.as_ptr()); }
    }

    pub fn flush_heads_changed(&mut self) {
        unsafe { weston_compositor_flush_heads_changed(self.as_ptr()); }
    }

    pub fn add_heads_changed_listener(&mut self, mut listener: mem::ManuallyDrop<Box<WlListener<CompositorRef>>>) {
        unsafe { weston_compositor_add_heads_changed_listener(self.as_ptr(), &mut listener.wll); }
    }

    pub fn iterate_heads(&mut self) -> HeadIterator {
        HeadIterator {
            compositor: self,
            head: ptr::null_mut(),
        }
    }

    pub fn create_output_with_head(&mut self, head: &mut HeadRef) -> Option<Output> {
        let ptr = unsafe { weston_compositor_create_output_with_head(self.as_ptr(), head.as_ptr()) };
        if ptr.is_null() {
            return None
        }
        Some(unsafe { Output::from_ptr(ptr) })
    }

    pub fn wake(&mut self) {
        unsafe { weston_compositor_wake(self.as_ptr()); }
    }

    pub fn shutdown(&mut self) {
        unsafe { weston_compositor_shutdown(self.as_ptr()); }
    }

    pub fn add_key_binding<'comp, F: FnMut(&mut KeyboardRef, &libc::timespec, u32)>(&'comp mut self, key: u32, modifier: KeyboardModifier, handler: &'comp F) {
        unsafe { weston_compositor_add_key_binding(self.as_ptr(), key, modifier.bits(), Some(run_key_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }

    pub fn add_modifier_binding<'comp, F: FnMut(&mut KeyboardRef, KeyboardModifier)>(&'comp mut self, modifier: KeyboardModifier, handler: &'comp F) {
        unsafe { weston_compositor_add_modifier_binding(self.as_ptr(), modifier.bits(), Some(run_modifier_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }

    pub fn add_button_binding<'comp, F: FnMut(&mut PointerRef, &libc::timespec, u32)>(&'comp mut self, button: u32, modifier: KeyboardModifier, handler: &'comp F) {
        unsafe { weston_compositor_add_button_binding(self.as_ptr(), button, modifier.bits(), Some(run_button_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }

    pub fn add_touch_binding<'comp, F: FnMut(&mut TouchRef, &libc::timespec)>(&'comp mut self, modifier: KeyboardModifier, handler: &'comp F) {
        unsafe { weston_compositor_add_touch_binding(self.as_ptr(), modifier.bits(), Some(run_touch_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }

    pub fn add_axis_binding<'comp, F: FnMut(&mut PointerRef, &libc::timespec, PointerAxisEvent)>(&'comp mut self, axis: Axis, modifier: KeyboardModifier, handler: &'comp F) {
        unsafe { weston_compositor_add_axis_binding(self.as_ptr(), axis.to_raw(), modifier.bits(), Some(run_axis_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }

    pub fn add_debug_binding<'comp, F: FnMut(&mut KeyboardRef, &libc::timespec, u32)>(&'comp mut self, key: u32, handler: &'comp F) {
        unsafe { weston_compositor_add_debug_binding(self.as_ptr(), key, Some(run_key_binding::<F>), handler as *const _ as *mut libc::c_void); }
    }
}
