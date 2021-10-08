use std::collections::{hash_set, HashMap, HashSet};

use super::HelpTopic;

#[derive(Default)]
pub struct HelpRegistry {
    entries: HashMap<String, HelpTopic>,
    file_docs: HashMap<String, &'static str>,
    filenames: HashSet<String>,
}

impl HelpRegistry {
    pub fn get(&self, name: &String) -> Option<&HelpTopic> {
        self.entries.get(name)
    }

    pub fn has_filename(&self, name: &String) -> bool {
        self.filenames.contains(name)
    }

    pub fn filenames(&self) -> hash_set::Iter<String> {
        self.filenames.iter()
    }

    pub fn insert(&mut self, topic: HelpTopic) {
        self.filenames.insert(topic.filename.to_string());
        self.entries.insert(topic.topic.to_string(), topic);
    }

    pub fn insert_filename_doc(&mut self, filename: &str, doc: &'static str) {
        self.filenames.insert(filename.to_string());
        self.file_docs.insert(filename.to_string(), doc);
    }

    pub fn doc_for_file(&self, filename: &str) -> Option<&&'static str> {
        self.file_docs.get(filename)
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
