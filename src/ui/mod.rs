use crate::editing::text::TextLine;

pub trait UI {
    /// Measure how many visual lines the given TextLine renders into
    /// for a given width
    fn measure_text_height(&self, line: TextLine, width: u16) -> u16;

    fn render_app(&mut self, app: &mut crate::app::State)
    where
        Self: Sized;
}
