use crate::editing::text::TextLine;

#[derive(Clone, Debug)]
pub enum Widget {
    Space,
    Spread(Vec<Widget>),
    Literal(TextLine),
}
