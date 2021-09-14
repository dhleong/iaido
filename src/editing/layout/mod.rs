use crate::tui::Renderable;

use super::{window::Window, FocusDirection, Id, Resizable, Size};

pub mod conn;
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
    fn current_focus(&self) -> Option<Id>;
    fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_>;
    fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id>;
    fn first_focus(&self, direction: FocusDirection) -> Option<Id>;
    fn size(&self) -> Size;
    fn windows_count(&self) -> usize;

    fn by_id_for_split(&mut self, id: Id) -> Option<&mut Box<Window>> {
        self.by_id_mut(id)
    }

    fn into_splittable(&mut self) -> Option<Box<&mut dyn SplitableLayout>> {
        None
    }
}

pub trait SplitableLayout {
    fn len(&self) -> usize;
    fn hsplit(&mut self, current_id: Id, win: Box<Window>);
    fn vsplit(&mut self, current_id: Id, win: Box<Window>);
    fn close_window(&mut self, win_id: Id);
    fn replace_window(&mut self, win_id: Id, layout: Box<dyn Layout>);
}
