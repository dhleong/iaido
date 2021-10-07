mod format;

pub use format::format_help as format;

pub struct HelpTopic {
    pub topic: String,
}

impl From<String> for HelpTopic {
    fn from(topic: String) -> HelpTopic {
        HelpTopic { topic }
    }
}

impl From<&&str> for HelpTopic {
    fn from(topic: &&str) -> HelpTopic {
        HelpTopic {
            topic: topic.to_string(),
        }
    }
}
