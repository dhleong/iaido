use super::Renderable;
use crate::editing::tabpage::Tabpage;

impl Renderable for Tabpage {
    fn render(&self, display: &mut super::Display) {
        // TODO
        self.current_window().render(display);
    }
}
