use tui::{
    style::{Color, Style},
    text::Span,
};

use crate::{connection::ReadValue, editing::text::TextLine};
use crate::{declare_commands, input::KeymapContext};

declare_commands!(declare_colors {
    pub fn colors(context) {
        // basic colors
        context.state_mut().current_buffer_mut().append_line("".to_string());
        context.state_mut().current_buffer_mut().append(
            basic_colors_line(|c| Style::default().bg(c)).into()
        );
        context.state_mut().current_buffer_mut().append(
            basic_colors_line(|c| Style::default().fg(c)).into()
        );
        context.state_mut().current_buffer_mut().append_line("".to_string());

        // high 256
        context.state_mut().current_buffer_mut().append_line("".to_string());
        for i in (16u32..=255).step_by(36) {
            for j in i..(i + 36) {
                if j > 255 {
                    break;
                }

                context.state_mut().current_buffer_mut().append_value(ReadValue::Text(
                    Span::styled(
                        ".",
                        Style::default()
                            .fg(Color::Indexed(j as u8))
                            .bg(Color::Indexed(j as u8))
                    ).into()
                ));
            }
            context.state_mut().current_buffer_mut().append_value(ReadValue::Newline);
        }

        // RGB
        let steps_per_line = 72;
        let lines = 3;
        let steps = lines * steps_per_line;
        context.state_mut().current_buffer_mut().append_line("".to_string());

        for i in (0..steps).step_by(steps_per_line) {
            for j in i..(i+steps_per_line) {
                let r = 255 - ((j * 255) / steps);
                let mut g = (j * 510) / steps;
                let b = (j * 255) / steps;
                if g > 255 { g = 510 - g }
                context.state_mut().current_buffer_mut().append_value(ReadValue::Text(
                    Span::styled(
                        if j % 2 == 0 {"/"} else {"\\"},
                        Style::default()
                            .bg(Color::Rgb(r as u8, g as u8, b as u8))
                            .fg(Color::Rgb(255 - r as u8, 255 - g as u8, 255 - b as u8)),
                    ).into()
                ));
            }
            context.state_mut().current_buffer_mut().append_value(ReadValue::Newline);
        }

        Ok(())
    }
});

fn basic_colors_line(to_style: impl Fn(Color) -> Style) -> TextLine {
    let mut line = TextLine::default();
    fn to_text(i: u8) -> String {
        if i < 10 {
            format!("___{}", i)
        } else {
            format!("__{}", i)
        }
    }

    for i in 0..8 {
        line.0
            .push(Span::styled(to_text(i), to_style(Color::Indexed(i))));
    }

    line.0.push(Span::raw("  "));

    for i in 8..16 {
        line.0
            .push(Span::styled(to_text(i), to_style(Color::Indexed(i))));
    }

    return line;
}
