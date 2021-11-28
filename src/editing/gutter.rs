use crate::editing::text::TextLine;

pub struct Gutter {
    pub width: u8,
    pub get_content: Box<dyn Fn(Option<usize>) -> TextLine>,
}
