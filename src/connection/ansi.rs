use std::collections::VecDeque;

use tui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use vte::{Parser, Perform};

use crate::editing::text::TextLine;

use super::ReadValue;

pub struct AnsiPipeline {
    parser: Parser,
    performer: AnsiPerformer,
}

impl AnsiPipeline {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            performer: AnsiPerformer::new(),
        }
    }

    pub fn feed(&mut self, buf: &[u8], n: usize) {
        for byte in &buf[..n] {
            self.parser.advance(&mut self.performer, *byte);
        }
    }

    pub fn next(&mut self) -> Option<ReadValue> {
        return self.performer.next();
    }
}

struct SpanBuilder {
    content: Option<String>,
    style: Style,
}

impl SpanBuilder {
    fn new() -> Self {
        Self {
            content: None,
            style: Style::default(),
        }
    }

    pub fn push(&mut self, ch: char) {
        if let Some(ref mut content) = self.content {
            content.push(ch);
        } else {
            let mut content = String::default();
            content.push(ch);
            self.content = Some(content);
        }
    }

    pub fn take(&mut self) -> Option<Span<'static>> {
        if let Some(content) = self.content.take() {
            Some(Span::styled(content, self.style))
        } else {
            None
        }
    }
}

struct AnsiPerformer {
    buffer: VecDeque<ReadValue>,
    builder: SpanBuilder,
    current_line: Option<TextLine>,
}

impl AnsiPerformer {
    fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            builder: SpanBuilder::new(),
            current_line: None,
        }
    }

    fn next(&mut self) -> Option<ReadValue> {
        self.line_to_buffer();
        self.buffer.pop_front()
    }

    fn builder_to_line(&mut self) {
        if let Some(span) = self.builder.take() {
            if let Some(ref mut line) = self.current_line {
                line.0.push(span);
            } else {
                self.current_line = Some(TextLine::from(vec![span]));
            }
        }
    }

    fn line_to_buffer(&mut self) {
        self.builder_to_line();
        if let Some(line) = self.current_line.take() {
            self.buffer.push_back(ReadValue::Text(line));
        }
    }
}

impl Perform for AnsiPerformer {
    fn print(&mut self, c: char) {
        if c == '\n' {
            self.line_to_buffer();
            self.buffer.push_back(ReadValue::Newline);
        } else {
            self.builder.push(c);
        }
    }

    fn execute(&mut self, _byte: u8) {}

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
    }

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if action != 'm' {
            // 'm' means "color"
            return;
        }

        self.builder_to_line();

        let mut style = Style::default();
        for param in params {
            let v = param[0];
            style = match v {
                0 => Style::reset(),

                1 => style.add_modifier(Modifier::BOLD),
                2 => style.add_modifier(Modifier::DIM),
                3 => style.add_modifier(Modifier::ITALIC),
                4 => style.add_modifier(Modifier::UNDERLINED),
                5 => style.add_modifier(Modifier::SLOW_BLINK),
                6 => style.add_modifier(Modifier::RAPID_BLINK),
                7 => style.add_modifier(Modifier::REVERSED),
                8 => style.add_modifier(Modifier::HIDDEN),
                9 => style.add_modifier(Modifier::CROSSED_OUT),

                21 => style.remove_modifier(Modifier::BOLD),
                22 => style.remove_modifier(Modifier::DIM),
                23 => style.remove_modifier(Modifier::ITALIC),
                24 => style.remove_modifier(Modifier::UNDERLINED),
                25 => style.remove_modifier(Modifier::SLOW_BLINK),
                26 => style.remove_modifier(Modifier::RAPID_BLINK),
                27 => style.remove_modifier(Modifier::REVERSED),
                28 => style.remove_modifier(Modifier::HIDDEN),
                29 => style.remove_modifier(Modifier::CROSSED_OUT),

                30..=37 => style.fg(match v {
                    30 => Color::Black,
                    31 => Color::Red,
                    32 => Color::Green,
                    33 => Color::Yellow,
                    34 => Color::Blue,
                    35 => Color::Magenta,
                    36 => Color::Cyan,
                    37 => Color::White,
                    _ => panic!(),
                }),
                39 => style.clear_fg(),

                40..=47 => style.bg(match v {
                    40 => Color::Black,
                    41 => Color::Red,
                    42 => Color::Green,
                    43 => Color::Yellow,
                    44 => Color::Blue,
                    45 => Color::Magenta,
                    46 => Color::Cyan,
                    47 => Color::White,
                    _ => panic!(),
                }),
                49 => style.clear_bg(),

                // default nop:
                _ => style,
            };
        }
        self.builder.style = style;
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

trait IaidoStyle {
    fn clear_bg(self) -> Style;
    fn clear_fg(self) -> Style;
}

impl IaidoStyle for Style {
    fn clear_bg(mut self) -> Style {
        self.bg = None;
        self
    }

    fn clear_fg(mut self) -> Style {
        self.fg = None;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestablePipline {
        fn feed_str(&mut self, text: &str);
    }

    impl TestablePipline for AnsiPipeline {
        fn feed_str(&mut self, text: &str) {
            self.feed(text.as_bytes(), text.len());
        }
    }

    #[test]
    fn simple_color() {
        let mut pipe = AnsiPipeline::new();
        pipe.feed_str("\x1b[1;36mTake my \x1b[3;32mlove");
        assert_eq!(
            pipe.next().unwrap(),
            ReadValue::Text(TextLine::from(vec![
                Span::styled(
                    "Take my ",
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan)
                ),
                Span::styled(
                    "love",
                    Style::default()
                        .add_modifier(Modifier::ITALIC)
                        .fg(Color::Green)
                )
            ],))
        );
    }
}
