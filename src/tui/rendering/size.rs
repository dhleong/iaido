use tui::layout::Rect;

use crate::editing::Size;

impl Into<Rect> for Size {
    fn into(self) -> Rect {
        Rect::new(0, 0, self.w, self.h)
    }
}

impl From<Rect> for Size {
    fn from(rect: Rect) -> Self {
        Self {
            w: rect.width,
            h: rect.height,
        }
    }
}

impl From<(u16, u16)> for Size {
    fn from(size: (u16, u16)) -> Self {
        Self {
            w: size.0,
            h: size.1,
        }
    }
}
