use pulldown_cmark::{CowStr, Event, Parser, Tag};
use tui::style::{Modifier, Style};
use tui::text::Span;

use crate::editing::text::{TextLine, TextLines};

struct HelpFormatter {
    lines: TextLines,
    current_line: Option<TextLine>,
    style: Style,
}

impl Default for HelpFormatter {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            current_line: Some(Default::default()),
            style: Default::default(),
        }
    }
}

impl HelpFormatter {
    fn has_pending_line(&self) -> bool {
        !self.current_line.as_ref().unwrap().0.is_empty()
    }

    fn add_modifier(&mut self, modifier: Modifier) {
        self.style = self.style.add_modifier(modifier);
    }

    fn remove_modifier(&mut self, modifier: Modifier) {
        self.style = self.style.remove_modifier(modifier);
    }

    fn push_line(&mut self) {
        self.lines.lines.push(self.current_line.take().unwrap());
        self.current_line = Some(TextLine::default());
    }

    fn push_str(&mut self, text: &str) {
        self.current_line
            .as_mut()
            .unwrap()
            .0
            .push(Span::styled(text.to_string(), self.style));
    }

    fn push_text(&mut self, text: CowStr) {
        self.push_str(&text);
    }
}

pub fn help(text: String) -> TextLines {
    let mut formatter = HelpFormatter::default();
    let parser = Parser::new(&text);

    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Emphasis => formatter.add_modifier(Modifier::ITALIC),
                    Tag::Heading(_) => {
                        if formatter.has_pending_line() {
                            formatter.push_line();
                        }
                        formatter.add_modifier(Modifier::BOLD);
                    }
                    Tag::Item => {
                        formatter.push_line();
                        formatter.push_str(" - ");
                    }
                    _ => {} // ignore
                };
            }
            Event::End(tag) => {
                match tag {
                    Tag::Emphasis => formatter.remove_modifier(Modifier::ITALIC),
                    Tag::Heading(_) => {
                        formatter.remove_modifier(Modifier::BOLD);
                        formatter.push_line();
                    }

                    Tag::Paragraph => formatter.push_line(),
                    Tag::List(_) => formatter.push_line(),

                    _ => {} // ignore
                };
            }

            Event::Text(text) => formatter.push_text(text),

            Event::HardBreak => {
                formatter.push_line();
            }
            _ => {} // Ignore, otherwise
        }
    }

    if formatter.has_pending_line() {
        formatter.push_line();
    }

    return formatter.lines;
}
