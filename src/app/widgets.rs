use crate::editing::text::TextLine;

pub enum Widget {
    Space,
    Spread(Vec<Widget>),
    Literal(TextLine),
}
