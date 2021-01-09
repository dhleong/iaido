use super::Renderable;
use crate::editing::tabpage::Tabpage;

impl Renderable for Tabpage {
    fn render<'a>(&self, app: &'a crate::App, display: &mut super::Display<'a>) {
        // TODO
        self.current_window().render(app, display);
    }
}
