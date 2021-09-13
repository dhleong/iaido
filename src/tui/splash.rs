use tui::layout::Rect;
use tui::widgets::Paragraph;
use tui::widgets::Widget;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
};

use super::rendering::display::Display;
use crate::editing::Size;

pub fn render(display: &mut Display) {
    let Size { w, h } = display.size;

    let widget_height = 3;
    let widget = Paragraph::new(vec![
        Spans::from("iaido"),
        Spans::from(""),
        Spans::from(vec![
            Span::from("type  :q"),
            Span::styled("<Enter>", Style::default().fg(Color::Magenta)),
            Span::from("  to exit"),
        ]),
    ])
    .alignment(Alignment::Center);

    widget.render(
        Rect::new(0, h / 2 - widget_height / 2, w, widget_height),
        &mut display.buffer,
    );
}
