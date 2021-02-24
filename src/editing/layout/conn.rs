use genawaiter::{sync::gen, yield_};

use crate::editing::{window::Window, FocusDirection, Id, Resizable, Size};

use super::Layout;

pub struct ConnLayout {
    pub output: Box<Window>,
    pub input: Box<Window>,
}

impl Layout for ConnLayout {
    fn by_id(&self, id: Id) -> Option<&Box<Window>> {
        if self.output.id == id {
            Some(&self.output)
        } else if self.input.id == id {
            Some(&self.input)
        } else {
            None
        }
    }

    fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>> {
        if self.output.id == id {
            Some(&mut self.output)
        } else if self.input.id == id {
            Some(&mut self.input)
        } else {
            None
        }
    }

    fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_> {
        Box::new(
            gen!({
                if self.output.buffer == buffer_id {
                    yield_!(&mut self.output);
                } else if self.input.buffer == buffer_id {
                    yield_!(&mut self.input);
                }
            })
            .into_iter(),
        )
    }

    fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id> {
        match direction {
            FocusDirection::Down if current_id == self.output.id => Some(self.input.id),
            FocusDirection::Up if current_id == self.input.id => Some(self.output.id),
            _ => None,
        }
    }

    fn first_focus(&self, direction: FocusDirection) -> Option<Id> {
        match direction {
            FocusDirection::Up | FocusDirection::Left => Some(self.input.id),
            FocusDirection::Right | FocusDirection::Down => Some(self.output.id),
        }
    }

    fn size(&self) -> Size {
        Size {
            w: self.output.size.w,
            h: self.output.size.h + self.input.size.h,
        }
    }
}

impl Resizable for ConnLayout {
    fn resize(&mut self, new_size: Size) {
        self.output.resize(Size {
            w: new_size.w,
            h: new_size.h - 1,
        });
        self.input.resize(Size {
            w: new_size.w,
            h: 1,
        });
    }
}
