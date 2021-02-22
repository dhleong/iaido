use genawaiter::{sync::gen, yield_};

use super::{window::Window, FocusDirection, Id, Resizable, Size};

pub enum LayoutEntry {
    Window(Box<Window>),
    Layout(Box<Layout>),
}

impl LayoutEntry {
    fn contains_window(&self, win_id: Id) -> bool {
        match self {
            &LayoutEntry::Window(ref win) => win.id == win_id,
            &LayoutEntry::Layout(ref lyt) => lyt.by_id(win_id).is_some(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

pub struct Layout {
    pub direction: LayoutDirection,
    pub entries: Vec<LayoutEntry>,
    pub primary_size: u16,
    pub cross_size: u16,
}

impl Layout {
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

    pub fn by_id(&self, id: Id) -> Option<&Box<Window>> {
        for entry in &self.entries {
            match entry {
                LayoutEntry::Window(win) if win.id == id => return Some(&win),
                LayoutEntry::Layout(lyt) => {
                    if let Some(win) = lyt.by_id(id) {
                        return Some(win);
                    }
                }
                _ => continue,
            }
        }
        None
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>> {
        for entry in &mut self.entries {
            match entry {
                LayoutEntry::Window(win) if win.id == id => return Some(win),
                LayoutEntry::Layout(lyt) => {
                    if let Some(win) = lyt.by_id_mut(id) {
                        return Some(win);
                    }
                }
                _ => continue,
            }
        }
        None
    }

    pub fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_> {
        Box::new(
            gen!({
                for entry in &mut self.entries {
                    match entry {
                        LayoutEntry::Window(win) => {
                            if win.id == buffer_id {
                                yield_!(win);
                            }
                        }

                        LayoutEntry::Layout(lyt) => {
                            for win in lyt.windows_for_buffer(buffer_id) {
                                yield_!(win);
                            }
                        }

                        _ => {}
                    }
                }
            })
            .into_iter(),
        )
    }

    pub fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id> {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.contains_window(current_id))
        {
            if let LayoutEntry::Layout(ref lyt) = self.entries.get(index).unwrap() {
                if let Some(next) = lyt.next_focus(current_id, direction) {
                    return Some(next);
                }
            }

            let mut next_index = index;
            loop {
                next_index = match direction {
                    FocusDirection::Up | FocusDirection::Left => {
                        if next_index == 0 {
                            return None;
                        }

                        next_index - 1
                    }
                    FocusDirection::Down | FocusDirection::Right => {
                        if next_index == self.entries.len() - 1 {
                            return None;
                        }

                        next_index + 1
                    }
                };

                if let Some(id) = match self.entries.get(next_index).unwrap() {
                    &LayoutEntry::Layout(ref lyt) => lyt.first_focus(direction),
                    &LayoutEntry::Window(ref win) => Some(win.id),
                } {
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
            match entry {
                &LayoutEntry::Layout(ref lyt) => lyt.first_focus(direction),
                &LayoutEntry::Window(ref win) => Some(win.id),
            }
        } else {
            None
        }
    }

    pub fn hsplit(&mut self, current_id: Id, win: Box<Window>) {
        self.split(
            current_id,
            win,
            LayoutDirection::Vertical,
            Layout::vertical(),
        );
    }

    pub fn vsplit(&mut self, current_id: Id, win: Box<Window>) {
        self.split(
            current_id,
            win,
            LayoutDirection::Horizontal,
            Layout::horizontal(),
        );
    }

    fn split(
        &mut self,
        current_id: Id,
        win: Box<Window>,
        direction: LayoutDirection,
        mut new_layout: Layout,
    ) {
        if self.direction == direction {
            self.entries.push(LayoutEntry::Window(win));
            self.resize(self.size());
            return;
        }

        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.contains_window(current_id))
        {
            match self.entries.remove(index) {
                LayoutEntry::Window(old_win) => {
                    new_layout.entries.push(LayoutEntry::Window(old_win));
                    new_layout.entries.push(LayoutEntry::Window(win));
                    self.entries
                        .insert(index, LayoutEntry::Layout(Box::new(new_layout)));
                    return;
                }

                LayoutEntry::Layout(mut lyt) => {
                    // put it back:
                    lyt.vsplit(current_id, win);
                    self.entries.insert(index, LayoutEntry::Layout(lyt));
                    return;
                }
            }
        }
    }

    pub fn size(&self) -> Size {
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

impl Resizable for Layout {
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
            let available = Size {
                h: primary_split + my_extra,
                w: self.cross_size,
            };
            match entry {
                LayoutEntry::Window(win) => win.resize(available),
                LayoutEntry::Layout(lyt) => lyt.resize(available),
            }
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
            let mut layout = Layout::vertical();
            layout
                .entries
                .push(LayoutEntry::Window(Box::new(Window::new(0, 0))));
            layout
                .entries
                .push(LayoutEntry::Window(Box::new(Window::new(1, 1))));
            layout
                .entries
                .push(LayoutEntry::Window(Box::new(Window::new(2, 2))));
            assert_eq!(Some(1), layout.next_focus(2, FocusDirection::Up));
            assert_eq!(Some(0), layout.next_focus(1, FocusDirection::Up));
            assert_eq!(None, layout.next_focus(0, FocusDirection::Up));
        }

        #[test]
        fn up_past_nested() {
            let mut layout = Layout::vertical();
            let mut a = Layout::horizontal();
            a.entries
                .push(LayoutEntry::Window(Box::new(Window::new(0, 0))));
            let mut b = Layout::horizontal();
            b.entries
                .push(LayoutEntry::Window(Box::new(Window::new(1, 1))));
            let mut c = Layout::horizontal();
            c.entries
                .push(LayoutEntry::Window(Box::new(Window::new(2, 2))));

            layout.entries.push(LayoutEntry::Layout(Box::new(a)));
            layout.entries.push(LayoutEntry::Layout(Box::new(b)));
            layout.entries.push(LayoutEntry::Layout(Box::new(c)));

            assert_eq!(Some(1), layout.next_focus(2, FocusDirection::Up));
            assert_eq!(Some(0), layout.next_focus(1, FocusDirection::Up));
            assert_eq!(None, layout.next_focus(0, FocusDirection::Up));
        }
    }
}
