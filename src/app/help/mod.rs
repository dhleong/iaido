mod format;
pub mod registry;

pub use format::format_help as format;

pub struct HelpQuery {
    pub query: String,
}

impl From<&&str> for HelpQuery {
    fn from(query: &&str) -> Self {
        Self {
            query: query.to_string(),
        }
    }
}

pub struct HelpTopic {
    pub filename: &'static str,
    pub topic: &'static str,
    pub doc: &'static str,
}
