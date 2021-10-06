use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use tui::style::{Color, Modifier, Style};
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

    fn push_span(&mut self, span: Span<'static>) {
        self.current_line.as_mut().unwrap().0.push(span);
    }

    fn push_str(&mut self, text: &str) {
        self.push_span(Span::styled(text.to_string(), self.style));
    }

    fn push_text(&mut self, text: CowStr) {
        self.push_str(&text);
    }

    fn push_keys(&mut self, text: CowStr) {
        let style = Style::default().fg(Color::Magenta);
        self.push_span(Span::styled(text.to_string(), style));
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Emphasis => self.add_modifier(Modifier::ITALIC),
            Tag::Heading(_) => {
                if self.has_pending_line() {
                    self.push_line();
                }
                self.add_modifier(Modifier::BOLD);
            }
            Tag::Item => self.push_str(" - "),
            Tag::List(_) => self.push_line(),
            Tag::Paragraph => self.push_line(),

            Tag::CodeBlock(kind) => {
                match kind {
                    CodeBlockKind::Fenced(_) => self.add_modifier(Modifier::DIM),
                    CodeBlockKind::Indented => {
                        self.push_line();
                        self.add_modifier(Modifier::DIM);
                    }
                };
            }

            Tag::Link(link_type, url, _) => {
                match link_type {
                    pulldown_cmark::LinkType::Autolink => {
                        self.push_keys(url);
                    }

                    _ => {}
                };
            }
            _ => {
                panic!("Unexpected tag: {:?}", tag);
            } // ignore
        };
    }

    fn end_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Emphasis => self.remove_modifier(Modifier::ITALIC),
            Tag::Heading(_) => {
                self.remove_modifier(Modifier::BOLD);
                self.push_line();
            }

            Tag::Paragraph => self.push_line(),
            Tag::Item => self.push_line(),
            Tag::List(_) => self.push_line(),

            Tag::CodeBlock(kind) => {
                match kind {
                    CodeBlockKind::Fenced(_) => self.remove_modifier(Modifier::DIM),
                    _ => {}
                };
            }

            _ => {} // ignore
        };
    }
}

pub fn help(text: String) -> TextLines {
    let mut formatter = HelpFormatter::default();
    let options = Options::empty();
    let parser = Parser::new_ext(&text, options);

    for event in parser {
        match event {
            Event::Start(tag) => formatter.start_tag(tag),
            Event::End(tag) => formatter.end_tag(tag),

            Event::Text(text) => formatter.push_text(text),
            Event::Html(html) => formatter.push_keys(html),
            Event::Code(text) => {
                formatter.add_modifier(Modifier::DIM);
                formatter.push_text(text);
                formatter.remove_modifier(Modifier::DIM);
            }

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
