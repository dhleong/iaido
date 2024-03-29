pub mod bufwin;
pub mod dispatcher;
pub mod help;
pub mod jobs;
pub mod looper;
pub mod popup;
pub mod prompt;
pub mod registers;
pub mod state;
pub mod widgets;
pub mod winsbuf;

use crate::ui::UI;
pub use state::AppState as State;

pub struct App<T: UI> {
    pub state: State,
    pub ui: T,
}

impl<T: UI> App<T> {
    pub fn new(state: State, ui: T) -> Self {
        Self { state, ui }
    }

    pub fn render(&mut self) {
        let wanted_redraw = self.state.requested_redraw;

        self.state.on_pre_draw();
        self.ui.render_app(&mut self.state);
        self.state.requested_redraw = false;

        if wanted_redraw {
            self.state.clear_echo();
        }
    }
}
