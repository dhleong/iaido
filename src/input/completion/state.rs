use crate::input::{maps::KeyHandlerContext, KeymapContext};

use super::{Completer, Completion, CompletionContext};

pub struct CompletionState {
    pub original: String,
    completions: Option<Box<dyn Iterator<Item = Completion>>>,
    current: Option<Completion>,
    history: Vec<Completion>,
    index: usize,
}

impl CompletionState {
    pub fn new<C: 'static + Completer, CTX>(
        completer: C,
        ctx: &mut KeyHandlerContext<CTX>,
    ) -> Self {
        let context: CompletionContext = ctx.state_mut().into();
        let original = context.word();
        return Self::from_completions(
            original.to_string(),
            Box::new(completer.suggest(ctx.state(), context)),
        );
    }

    pub fn from_completions(
        original: String,
        completions: Box<dyn Iterator<Item = Completion>>,
    ) -> Self {
        Self {
            original,
            completions: Some(completions),
            current: None,
            history: Vec::default(),
            index: 0,
        }
    }

    pub fn take_current(&mut self) -> Option<Completion> {
        self.current.take()
    }

    pub fn advance(&mut self) -> Option<Completion> {
        if let Some(completions) = &mut self.completions {
            if let Some(next) = completions.next() {
                return Some(next);
            }
        }

        self.completions = None;

        None
    }

    pub fn push_history(&mut self, prev: Option<Completion>, current: Option<Completion>) {
        if let Some(prev) = prev {
            self.history.push(prev);
            self.index = self.history.len()
        }
        self.current = current;
    }

    pub fn apply_next<C: KeymapContext>(&mut self, ctx: &mut C) {
        // bit of a dance: we actually take ownership temporarily
        // and return it after
        let prev = self.take_current();
        if let Some(next) = self.advance() {
            ctx.state_mut()
                .current_buffer_mut()
                .apply_completion(prev.as_ref(), Some(&next));
            ctx.state_mut()
                .current_window_mut()
                .apply_completion(Some(&next));
            self.push_history(prev, Some(next));
        }
    }
}
