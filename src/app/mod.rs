pub mod bufwin;
pub mod looper;
pub mod prompt;
pub mod state;

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
        self.ui.render_app(&mut self.state);
    }
}
