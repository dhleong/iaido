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

    pub fn take_current(&mut self) -> Option<Completion> {
        self.current.take()
    }

    pub fn advance(&mut self) -> Option<Completion> {
        self.completions.next()
    }

    pub fn push_history(&mut self, prev: Option<Completion>, current: Option<Completion>) {
        if let Some(prev) = prev {
            self.history.push(prev);
            self.index = self.history.len()
        }
        self.current = current;
    }
}
