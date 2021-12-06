pub mod backtrace;

use std::{io, time::Duration};

use crate::{editing::text::TextLine, input::Key};

#[derive(Clone, Copy)]
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

pub trait UiEvents {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<UiEvent>>;
    fn next_event(&mut self) -> io::Result<UiEvent>;
}
