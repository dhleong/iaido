use crate::editing::motion::MotionContext;

use super::Completion;

pub struct CompletionState {
    completions: Box<dyn Iterator<Item = Completion>>,
    current: Option<Completion>,
    history: Vec<Completion>,
    index: usize,
}

impl CompletionState {
    pub fn new(mut completions: Box<dyn Iterator<Item = Completion>>) -> Self {
        let current = completions.next();
        Self {
            completions,
            current,
            history: Vec::default(),
            index: 0,
        }
    }

    pub fn current(&self) -> Option<&Completion> {
        self.current.as_ref()
    }
}
