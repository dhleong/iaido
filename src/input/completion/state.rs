use std::rc::Rc;

use crate::{app::popup::PopupMenu, editing::motion::MotionContext, input::KeymapContext};

use super::{CompletableContext, Completer, Completion, CompletionContext};

pub struct BoxedCompleter {
    delegate: Rc<dyn Completer>,
}

impl From<Rc<dyn Completer>> for BoxedCompleter {
    fn from(delegate: Rc<dyn Completer>) -> Self {
        BoxedCompleter { delegate }
    }
}

impl Completer for BoxedCompleter {
    fn suggest(
        &self,
        app: Box<&dyn CompletableContext>,
        context: CompletionContext,
    ) -> super::BoxedSuggestions {
        self.delegate.suggest(app, context)
    }
}

pub struct CompletionState {
    completions: Option<Box<dyn Iterator<Item = Completion>>>,
    history: Vec<Completion>,
    index: usize,
}

impl CompletionState {
    pub fn new<C: Completer, CTX: KeymapContext>(completer: C, ctx: &mut CTX) -> Self {
        let context: CompletionContext = ctx.state_mut().into();
        return Self::from_context(completer, ctx.state(), context);
    }

    pub fn from_context<C: Completer, CTX: CompletableContext>(
        completer: C,
        app: &CTX,
        context: CompletionContext,
    ) -> Self {
        let original = context.word();
        return Self::from_completions(
            context.create_completion(original.to_string()),
            Box::new(completer.suggest(Box::new(app), context)),
        );
    }

    pub fn from_completions(
        original: Completion,
        completions: Box<dyn Iterator<Item = Completion>>,
    ) -> Self {
        Self {
            completions: Some(completions),
            history: vec![original],
            index: 1,
        }
    }

    pub fn to_pum(&self) -> Option<PopupMenu> {
        // NOTE: The first "history" item is the raw input; that's the source
        // of the extra 1 everywhere in here:
        if self.completions.is_none() && self.history.len() < 2 {
            return None;
        }

        let contents: Vec<String> = self
            .history
            .iter()
            .skip(1)
            .map(|completion| completion.replacement.to_string())
            .collect();

        let horizontal_offset =
            if let Some(item) = self.history.get(self.index.checked_sub(1).unwrap_or(0)) {
                item.replacement_end().col - item.start
            } else {
                0usize
            };

        Some(PopupMenu {
            contents,
            cursor: self.index.checked_sub(2),
            horizontal_offset,
        })
    }

    pub fn apply_next<C: MotionContext>(&mut self, ctx: &mut C) {
        // bit of a dance: we actually take ownership temporarily
        // and return it after
        let current_index = self.index - 1;
        let prev = self.take_current();
        if let Some(next) = self.advance() {
            ctx.buffer_mut().apply_completion(&prev, &next);
            ctx.window_mut().apply_completion(&next);
            self.history.insert(current_index, next);
        }
        self.history.insert(current_index, prev);
    }

    pub fn apply_prev<C: MotionContext>(&mut self, ctx: &mut C) {
        // bit of a dance: we actually take ownership temporarily
        // and return it after
        let current_index = self.index - 1;
        let current = self.take_current();
        if let Some(ref prev) = self.back() {
            ctx.buffer_mut().apply_completion(&current, &prev);
            ctx.window_mut().apply_completion(&prev);
        }
        self.history.insert(current_index, current);
    }

    fn take_current(&mut self) -> Completion {
        self.history.remove(self.index - 1)
    }

    fn advance(&mut self) -> Option<Completion> {
        if self.index <= self.history.len() {
            let result = self.history.remove(self.index - 1);
            self.index += 1;
            return Some(result);
        }

        if let Some(completions) = &mut self.completions {
            if let Some(next) = completions.next() {
                self.index += 1;
                return Some(next);
            }
        }

        self.completions = None;

        None
    }

    fn back(&mut self) -> Option<&Completion> {
        if self.index <= 1 {
            return None;
        }
        self.index -= 1;
        return self.history.get(self.index - 1);
    }
}

#[cfg(test)]
mod tests {
    use genawaiter::{rc::gen, yield_};

    use crate::editing::motion::tests::{window, TestWindow};

    use super::*;

    crate::declare_simple_completer!(TestCompleter {
        gen!({
            yield_!("love".to_string());
            yield_!("land".to_string());
            yield_!("where".to_string());
        })
    });

    fn completion_state(win: &mut TestWindow) -> CompletionState {
        let context: CompletionContext = win.into();
        CompletionState::from_context(TestCompleter, win, context)
    }

    #[test]
    fn apply_next() {
        let mut win = window("take my |");
        win.set_inserting(true);
        win.assert_visual_match("take my |");

        let mut state = completion_state(&mut win);
        state.apply_next(&mut win);
        win.assert_visual_match("take my love|");

        state.apply_next(&mut win);
        win.assert_visual_match("take my land|");

        state.apply_next(&mut win);
        win.assert_visual_match("take my where|");

        // don't explode:
        state.apply_next(&mut win);
        win.assert_visual_match("take my where|");
    }

    #[test]
    fn apply_prev_and_next() {
        let mut win = window("take my |");
        win.set_inserting(true);
        let mut state = completion_state(&mut win);
        state.apply_next(&mut win);
        state.apply_next(&mut win);
        state.apply_next(&mut win);
        win.assert_visual_match("take my where|");

        state.apply_prev(&mut win);
        win.assert_visual_match("take my land|");

        state.apply_prev(&mut win);
        win.assert_visual_match("take my love|");

        state.apply_prev(&mut win);
        win.assert_visual_match("take my |");

        // don't explode:
        state.apply_prev(&mut win);
        win.assert_visual_match("take my |");

        // and... back forward
        state.apply_next(&mut win);
        win.assert_visual_match("take my love|");

        state.apply_next(&mut win);
        win.assert_visual_match("take my land|");

        state.apply_next(&mut win);
        win.assert_visual_match("take my where|");
    }
}
