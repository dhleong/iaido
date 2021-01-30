use tui::layout::Rect;

use crate::{editing::Buffer, tui::Display};

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
