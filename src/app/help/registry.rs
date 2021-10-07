use std::collections::{hash_set, HashMap, HashSet};

use super::HelpTopic;

#[derive(Default)]
pub struct HelpRegistry {
    entries: HashMap<String, HelpTopic>,
    filenames: HashSet<String>,
}

impl HelpRegistry {
    pub fn get(&self, name: &String) -> Option<&HelpTopic> {
        self.entries.get(name)
    }

    pub fn filenames(&self) -> hash_set::Iter<String> {
        self.filenames.iter()
    }

    pub fn insert(&mut self, topic: HelpTopic) {
        self.filenames.insert(topic.filename.to_string());
        self.entries.insert(topic.topic.to_string(), topic);
    }

    pub fn entries_for_file(&self, filename: &str) -> Vec<&HelpTopic> {
        let mut entries: Vec<&HelpTopic> = self
            .entries
            .values()
            .filter(|topic| topic.filename == filename)
            .collect();
        entries.sort_by_key(|topic| topic.topic);
        entries
    }
}
