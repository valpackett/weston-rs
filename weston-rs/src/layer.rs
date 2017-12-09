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
use ::compositor::Compositor;
use ::view::View;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LayerPosition {
    Hidden = weston_layer_position_WESTON_LAYER_POSITION_HIDDEN,
    Background = weston_layer_position_WESTON_LAYER_POSITION_BACKGROUND,
    BottomUi = weston_layer_position_WESTON_LAYER_POSITION_BOTTOM_UI,
    Normal = weston_layer_position_WESTON_LAYER_POSITION_NORMAL,
    Ui = weston_layer_position_WESTON_LAYER_POSITION_UI,
    Fullscreen = weston_layer_position_WESTON_LAYER_POSITION_FULLSCREEN,
    TopUi = weston_layer_position_WESTON_LAYER_POSITION_TOP_UI,
    Lock = weston_layer_position_WESTON_LAYER_POSITION_LOCK,
    Cursor = weston_layer_position_WESTON_LAYER_POSITION_CURSOR,
    Fade = weston_layer_position_WESTON_LAYER_POSITION_FADE,
}

pub struct Layer<'comp> {
    layer: Box<weston_layer>,
    phantom: marker::PhantomData<&'comp Compositor>,
}

impl<'comp> Layer<'comp> {
    pub fn new(compositor: &'comp Compositor) -> Layer<'comp> {
        let mut result = Layer {
            layer: Box::new(unsafe { mem::zeroed() }),
            phantom: marker::PhantomData,
        };
        unsafe { weston_layer_init(&mut *result.layer, compositor.ptr()); }
        result
    }

    pub fn set_position(&mut self, position: LayerPosition) {
        unsafe { weston_layer_set_position(&mut *self.layer, position as weston_layer_position); }
    }

    pub fn entry_insert(&mut self, view: &mut View) {
        unsafe { weston_layer_entry_insert(&mut (*self.layer).view_list, view.layer_link()); }
    }

    pub fn ptr(&mut self) -> *mut weston_layer {
        &mut *self.layer
    }
}
