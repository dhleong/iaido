use clap::crate_version;
use tui::layout::Rect;
use tui::style::Modifier;
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
    let var_style = Style::default().add_modifier(Modifier::BOLD);
    let enter_style = Style::default().fg(Color::Magenta);

    // TODO Maybe we can simplify the alignment and formatting of these hints?
    let lines = vec![
        Spans::from("iaido"),
        Spans::from(format!("version {}", crate_version!())),
        Spans::from(""),
        Spans::from(vec![
            Span::from("type  :connect "),
            Span::styled("<host>", var_style),
            Span::from(":"),
            Span::styled("<port>", var_style),
            Span::styled("<Enter>", enter_style),
            Span::from("  to connect"),
        ]),
        Spans::from(vec![
            Span::from("type  :q"),
            Span::styled("<Enter>", enter_style),
            Span::from("                      to exit   "),
        ]),
    ];

    let widget_height = lines.len() as u16;
    let widget = Paragraph::new(lines).alignment(Alignment::Center);

    // Render vertically centered
    let Size { w, h } = display.size;
    widget.render(
        Rect::new(0, h / 2 - widget_height / 2, w, widget_height),
        &mut display.buffer,
    );
}
