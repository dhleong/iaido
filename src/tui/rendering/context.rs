use tui::layout::Rect;

use crate::{
    editing::{buffers::Buffers, Buffer},
    tui::Display,
};

pub struct LayoutContext<'a> {
    pub buffers: Option<&'a Buffers>,
    pub buffer_override: Option<&'a Box<dyn Buffer>>,
}

impl<'a> LayoutContext<'a> {
    pub fn new(buffers: &'a Buffers) -> Self {
        Self {
            buffers: Some(&buffers),
            buffer_override: None,
        }
    }

    pub fn with_buffer(buffer: &'a Box<dyn Buffer>) -> Self {
        Self {
            buffers: None,
            buffer_override: Some(buffer),
        }
    }

    pub fn buffer(&self, id: usize) -> Option<&Box<dyn Buffer>> {
        if let Some(overridden) = self.buffer_override {
            Some(overridden)
        } else if let Some(buffers) = self.buffers {
            buffers.by_id(id)
        } else {
            panic!("Had neither buffers nor buffer")
        }
    }
}

pub struct RenderContext<'a> {
    pub app: &'a crate::app::State,
    pub display: &'a mut Display,
    pub area: Rect,
    pub buffer_override: Option<&'a Box<dyn Buffer>>,
}

impl<'a> RenderContext<'a> {
    pub fn new(app: &'a crate::app::State, display: &'a mut Display) -> Self {
        let area = display.size.into();
        Self {
            app,
            display,
            area,
            buffer_override: None,
        }
    }

    pub fn with_buffer(self, buffer_override: &'a Box<dyn Buffer>) -> Self {
        Self {
            app: self.app,
            display: self.display,
            area: self.area,
            buffer_override: Some(buffer_override),
        }
    }

    pub fn with_area(&mut self, new_area: Rect) -> RenderContext {
        RenderContext {
            app: self.app,
            display: self.display,
            area: new_area,
            buffer_override: self.buffer_override,
        }
    }
}
