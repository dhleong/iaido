use std::io;

use async_trait::async_trait;

use crate::{editing::text::TextLine, input::Key};

pub enum UiEvent {
    Redraw,
    Key(Key),
    // UiThreadFn(Box<dyn Fn() + Send>), // ?
}

pub trait UI {
    /// Measure how many visual lines the given TextLine renders into
    /// for a given width
    fn measure_text_height(&self, line: TextLine, width: u16) -> u16;

    fn render_app(&mut self, app: &mut crate::app::State)
    where
        Self: Sized;
}

#[async_trait]
pub trait UiEvents {
    async fn next_event(&mut self) -> Result<UiEvent, io::Error>;
}
