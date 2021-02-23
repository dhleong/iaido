use crate::tui::Renderable;

use super::{window::Window, FocusDirection, Id, Resizable, Size};

mod linear;
pub mod win;

pub use linear::{LayoutDirection, LinearLayout};

// NOTE: explicitly making every Layout implement Renderable is a bit
// of a bummer (the TUI crate leaks into here) but seems to be required
// by the refactor to make Layout a polymorphic trait---which is easily
// a win in itself, so... still net win.

pub trait Layout: Renderable + Resizable {
    fn by_id(&self, id: Id) -> Option<&Box<Window>>;
    fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>>;
    fn contains_window(&self, win_id: Id) -> bool {
        self.by_id(win_id).is_some()
    }
    fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_>;
    fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id>;
    fn first_focus(&self, direction: FocusDirection) -> Option<Id>;
    fn size(&self) -> Size;

    fn as_splittable(&self) -> Option<Box<&dyn SplitableLayout>> {
        None
    }
}

pub trait SplitableLayout {
    fn hsplit(&mut self, current_id: Id, win: Box<Window>);
    fn vsplit(&mut self, current_id: Id, win: Box<Window>);
}
