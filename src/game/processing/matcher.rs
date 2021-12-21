use crate::editing::text::TextLine;

pub struct Matcher {
    pub description: String,
}

impl Matcher {
    pub fn find(&self, _input: TextLine) -> Option<Match> {
        // TODO
        None
    }
}

pub struct Match {}

impl Match {
    pub fn group(&self, _name: &str) -> Option<TextLine> {
        // TODO
        None
    }
}
