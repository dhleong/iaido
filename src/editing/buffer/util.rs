use crate::editing::{
    motion::{MotionFlags, MotionRange},
    CursorPosition,
};

use super::Buffer;

pub enum LineRange {
    WholeLine,
    FromCol(usize, bool),
    ToCol(usize, bool),
    Precise(usize, usize, bool),
}

impl LineRange {
    pub fn resolve<T: Buffer>(&self, line_index: usize, buffer: &T) -> (usize, usize) {
        let width = buffer.get_line_width(line_index).unwrap_or(0);
        match self {
            LineRange::WholeLine => (0, width),
            &LineRange::FromCol(col, _) => (col, width),
            &LineRange::ToCol(col, _) => (0, col),
            &LineRange::Precise(start, end, _) => (start, end),
        }
    }

    pub fn is_whole_line<T: Buffer>(&self, line_index: usize, buffer: &T) -> bool {
        let width = buffer.get_line_width(line_index).unwrap_or(0);
        match self {
            LineRange::WholeLine => true,
            &LineRange::FromCol(col, linewise) if col == 0 => linewise,
            &LineRange::ToCol(col, linewise) if col == width => linewise,
            &LineRange::Precise(start, end, linewise) if (start, end) == (0, width) => linewise,
            _ => false,
        }
    }
}

pub fn motion_to_line_ranges(range: MotionRange) -> impl Iterator<Item = LineRange> {
    let MotionRange(
        CursorPosition {
            line: first_line,
            col: first_col,
        },
        CursorPosition {
            line: last_line,
            col: last_col,
        },
        flags,
    ) = range;

    let linewise = flags.contains(MotionFlags::LINEWISE) || last_line > first_line;
    (first_line..=last_line).map(move |line_index| {
        if line_index == first_line && line_index == last_line {
            // single line range:
            LineRange::Precise(first_col, last_col, linewise)
        } else if line_index == first_line {
            LineRange::FromCol(first_col, linewise)
        } else if line_index == last_line {
            LineRange::ToCol(last_col, linewise)
        } else {
            LineRange::WholeLine
        }
    })
}
