use genawaiter::{sync::gen, yield_};

use crate::editing::{window::Window, FocusDirection, Id, Resizable, Size};

use super::{win::WinLayout, Layout, SplitableLayout};

#[derive(Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

pub struct LinearLayout {
    pub direction: LayoutDirection,
    pub entries: Vec<Box<dyn Layout>>,
    pub primary_size: u16,
    pub cross_size: u16,
}

impl LinearLayout {
    pub fn horizontal() -> Self {
        Self {
            direction: LayoutDirection::Horizontal,
            entries: Vec::new(),
            primary_size: 0,
            cross_size: 0,
        }
    }

    pub fn vertical() -> Self {
        Self {
            direction: LayoutDirection::Vertical,
            entries: Vec::new(),
            primary_size: 0,
            cross_size: 0,
        }
    }

    pub fn add_window(&mut self, window: Box<Window>) {
        self.entries.push(Box::new(WinLayout::new(window)))
    }

    fn split(
        &mut self,
        current_id: Id,
        win: Box<Window>,
        direction: LayoutDirection,
        mut new_layout: LinearLayout,
    ) {
        if self.direction == direction {
            self.entries.push(Box::new(WinLayout::new(win)));
            self.resize(self.size());
            return;
        }

        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.contains_window(current_id))
        {
            let mut lyt = self.entries.swap_remove(index);
            let replacement = if let Some(splittable) = lyt.as_splittable() {
                match direction {
                    LayoutDirection::Horizontal => splittable.vsplit(current_id, win),
                    LayoutDirection::Vertical => splittable.hsplit(current_id, win),
                }
                lyt
            } else {
                new_layout.entries.push(lyt);
                new_layout.entries.push(Box::new(WinLayout::new(win)));
                Box::new(new_layout)
            };

            self.entries.push(replacement);
            if self.entries.len() > 1 {
                let last = self.entries.len() - 1;
                self.entries.swap(index, last);
            }
        }
    }
}

impl Layout for LinearLayout {
    fn by_id(&self, id: Id) -> Option<&Box<Window>> {
        for entry in &self.entries {
            if let Some(win) = entry.by_id(id) {
                return Some(win);
            }
        }
        None
    }

    fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>> {
        for entry in &self.entries {
            if let Some(win) = entry.by_id_mut(id) {
                return Some(win);
            }
        }
        None
    }

    fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_> {
        Box::new(
            gen!({
                for entry in &mut self.entries {
                    for win in entry.windows_for_buffer(buffer_id) {
                        yield_!(win);
                    }
                }
            })
            .into_iter(),
        )
    }

    fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.contains_window(current_id))
        {
            let lyt = self.entries.get(index).unwrap();
            if let Some(next) = lyt.next_focus(current_id, direction) {
                return Some(next);
            }

            let mut next_index = index;
            loop {
                next_index = match (self.direction, direction) {
                    (LayoutDirection::Vertical, FocusDirection::Up)
                    | (LayoutDirection::Horizontal, FocusDirection::Left) => {
                        if next_index == 0 {
                            return None;
                        }

                        next_index - 1
                    }
                    (LayoutDirection::Vertical, FocusDirection::Down)
                    | (LayoutDirection::Horizontal, FocusDirection::Right) => {
                        if next_index == self.entries.len() - 1 {
                            return None;
                        }

                        next_index + 1
                    }
                    _ => return None,
                };

                if let Some(id) = self.entries.get(next_index).unwrap().first_focus(direction) {
                    return Some(id);
                }
            }
        }

        None
    }

    fn first_focus(&self, direction: FocusDirection) -> Option<Id> {
        if let Some(entry) = match direction {
            FocusDirection::Up | FocusDirection::Left => self.entries.last(),
            FocusDirection::Right | FocusDirection::Down => self.entries.first(),
        } {
            entry.first_focus(direction)
        } else {
            None
        }
    }

    fn size(&self) -> Size {
        match self.direction {
            LayoutDirection::Horizontal => Size {
                w: self.primary_size,
                h: self.cross_size,
            },
            LayoutDirection::Vertical => Size {
                w: self.cross_size,
                h: self.primary_size,
            },
        }
    }
}

impl SplitableLayout for LinearLayout {
    fn hsplit(&mut self, current_id: Id, win: Box<Window>) {
        self.split(
            current_id,
            win,
            LayoutDirection::Vertical,
            LinearLayout::vertical(),
        );
    }

    fn vsplit(&mut self, current_id: Id, win: Box<Window>) {
        self.split(
            current_id,
            win,
            LayoutDirection::Horizontal,
            LinearLayout::horizontal(),
        );
    }
}

impl Resizable for LinearLayout {
    fn resize(&mut self, new_size: super::Size) {
        match self.direction {
            LayoutDirection::Vertical => {
                self.primary_size = new_size.h;
                self.cross_size = new_size.w;
            }
            LayoutDirection::Horizontal => {
                self.primary_size = new_size.w;
                self.cross_size = new_size.h;
            }
        };

        let count = self.entries.len() as u16;
        if count == 0 || self.primary_size == 0 {
            // nop
            return;
        }

        let borders = count - 1;
        let primary_split = (self.primary_size - borders) / count;
        let extra = self.primary_size - borders - (primary_split * count);
        for (i, entry) in &mut self.entries.iter_mut().enumerate() {
            // TODO can/should we try to maintain current ratios?
            let my_extra = if i == 0 { extra } else { 0 };
            let available = match self.direction {
                LayoutDirection::Vertical => Size {
                    h: primary_split + my_extra,
                    w: self.cross_size,
                },
                LayoutDirection::Horizontal => Size {
                    w: primary_split + my_extra,
                    h: self.cross_size,
                },
            };
            entry.resize(available);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod move_focus {
        use super::*;

        #[test]
        fn up_in_vertical() {
            let mut layout = LinearLayout::vertical();
            layout.add_window(Box::new(Window::new(0, 0)));
            layout.add_window(Box::new(Window::new(1, 1)));
            layout.add_window(Box::new(Window::new(2, 2)));
            assert_eq!(Some(1), layout.next_focus(2, FocusDirection::Up));
            assert_eq!(Some(0), layout.next_focus(1, FocusDirection::Up));
            assert_eq!(None, layout.next_focus(0, FocusDirection::Up));
        }

        #[test]
        fn up_past_nested() {
            let mut layout = LinearLayout::vertical();
            let mut a = LinearLayout::horizontal();
            a.add_window(Box::new(Window::new(0, 0)));
            let mut b = LinearLayout::horizontal();
            b.add_window(Box::new(Window::new(1, 1)));
            let mut c = LinearLayout::horizontal();
            c.add_window(Box::new(Window::new(2, 2)));

            layout.entries.push(Box::new(a));
            layout.entries.push(Box::new(b));
            layout.entries.push(Box::new(c));

            assert_eq!(Some(1), layout.next_focus(2, FocusDirection::Up));
            assert_eq!(Some(0), layout.next_focus(1, FocusDirection::Up));
            assert_eq!(None, layout.next_focus(0, FocusDirection::Up));
        }

        #[test]
        fn up_from_horizontal() {
            //    0
            // -------
            //  1 | 2
            let mut container = LinearLayout::vertical();
            let mut bottom = LinearLayout::horizontal();
            bottom.add_window(Box::new(Window::new(1, 1)));
            bottom.add_window(Box::new(Window::new(2, 2)));

            container.add_window(Box::new(Window::new(0, 0)));
            container.entries.push(Box::new(bottom));

            assert_eq!(Some(0), container.next_focus(2, FocusDirection::Up));
            assert_eq!(Some(0), container.next_focus(1, FocusDirection::Up));
            assert_eq!(None, container.next_focus(0, FocusDirection::Up));
        }
    }
}
