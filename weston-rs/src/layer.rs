use std::{mem, marker};
use libweston_sys::{
    weston_layer, weston_layer_init,
    weston_layer_position, weston_layer_set_position,
    weston_layer_position_WESTON_LAYER_POSITION_HIDDEN,
    weston_layer_position_WESTON_LAYER_POSITION_BACKGROUND,
    weston_layer_position_WESTON_LAYER_POSITION_BOTTOM_UI,
    weston_layer_position_WESTON_LAYER_POSITION_NORMAL,
    weston_layer_position_WESTON_LAYER_POSITION_UI,
    weston_layer_position_WESTON_LAYER_POSITION_FULLSCREEN,
    weston_layer_position_WESTON_LAYER_POSITION_TOP_UI,
    weston_layer_position_WESTON_LAYER_POSITION_LOCK,
    weston_layer_position_WESTON_LAYER_POSITION_CURSOR,
    weston_layer_position_WESTON_LAYER_POSITION_FADE,
    weston_layer_entry_insert
};
use foreign_types::{ForeignType, ForeignTypeRef};
use ::compositor::Compositor;
use ::view::View;

/// Layer order (higher value means higher in the stack).
///
/// Values based on well-known concepts in a classic desktop environment are provided in this
/// module, but you don't have to use them.
pub type LayerPosition = u32;

pub const POSITION_HIDDEN: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_HIDDEN;
pub const POSITION_BACKGROUND: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_BACKGROUND;
pub const POSITION_BOTTOM_UI: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_BOTTOM_UI;
pub const POSITION_NORMAL: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_NORMAL;
pub const POSITION_UI: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_UI;
pub const POSITION_FULLSCREEN: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_FULLSCREEN;
pub const POSITION_TOP_UI: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_TOP_UI;
pub const POSITION_LOCK: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_LOCK;
pub const POSITION_CURSOR: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_CURSOR;
pub const POSITION_FADE: LayerPosition = weston_layer_position_WESTON_LAYER_POSITION_FADE;

fn drop_layer(ptr: *mut weston_layer) {
    let _ = unsafe { Box::from_raw(ptr) };
}

foreign_type! {
    type CType = weston_layer;
    fn drop = drop_layer;
    pub struct Layer;
    pub struct LayerRef;
}

unsafe impl Sync for Layer {}

impl Layer {
    pub fn new(compositor: &Compositor) -> Layer {
        let ptr = Box::into_raw(Box::new(unsafe { mem::zeroed() }));
        unsafe {
            weston_layer_init(ptr, compositor.as_ptr());
            Layer::from_ptr(ptr)
        }
    }
}

impl LayerRef {
    pub fn set_position(&mut self, position: LayerPosition) {
        unsafe { weston_layer_set_position(self.as_ptr(), position as weston_layer_position); }
    }

    pub fn entry_insert(&mut self, view: &mut View) {
        unsafe { weston_layer_entry_insert(&mut (*self.as_ptr()).view_list, view.layer_link()); }
    }
}
