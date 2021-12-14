use std::convert::TryInto;

use crate::editing::Size;

pub struct PopupMenu {
    pub contents: Vec<String>,

    /// Selected index within `contents`, if any
    pub cursor: Option<usize>,

    /// Offset towards the start of the line that the PUM should be
    /// offset relative to the cursor
    pub horizontal_offset: usize,
}

impl PopupMenu {
    pub fn measure(&self, size: Size) -> Size {
        let width = std::cmp::min(
            size.w.checked_sub(2).unwrap_or(1),
            self.contents
                .iter()
                .map(|item| item.len())
                .max()
                .unwrap_or(0)
                .try_into()
                .unwrap_or(1),
        );

        let height = std::cmp::min(
            (size.h / 2).checked_sub(2).unwrap_or(1),
            self.contents.len().try_into().unwrap_or(1),
        );

        (width, height).into()
    }
}
