use genawaiter::{sync::gen, yield_};

use super::{window::Window, Id, Resizable, Size};

pub enum LayoutEntry {
    Window(Box<Window>),
    Layout(Box<Layout>),
}

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

    pub fn split(&mut self, win: Box<Window>) {
        self.entries.push(LayoutEntry::Window(win));
        self.resize(self.size())
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
