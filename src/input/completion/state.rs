use crate::{editing::motion::MotionContext, input::KeymapContext};

use super::{CompletableContext, Completer, Completion, CompletionContext};

pub struct CompletionState {
    completions: Option<Box<dyn Iterator<Item = Completion>>>,
    history: Vec<Completion>,
    index: usize,
}

impl CompletionState {
    pub fn new<C: 'static + Completer, CTX: KeymapContext>(completer: C, ctx: &mut CTX) -> Self {
        let context: CompletionContext = ctx.state_mut().into();
        return Self::from_context(completer, ctx.state(), context);
    }

    pub fn from_context<C: 'static + Completer, CTX: CompletableContext>(
        completer: C,
        app: &CTX,
        context: CompletionContext,
    ) -> Self {
        let original = context.word();
        return Self::from_completions(
            context.create_completion(original.to_string()),
            Box::new(completer.suggest(app, context)),
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

    pub fn apply_next<C: MotionContext>(&mut self, ctx: &mut C) {
        // bit of a dance: we actually take ownership temporarily
        // and return it after
        let prev = self.take_current();
        if let Some(next) = self.advance() {
            ctx.buffer_mut().apply_completion(&prev, &next);
            ctx.window_mut().apply_completion(&next);
            self.push_history(prev);
            self.push_history(next);
        } else {
            self.push_history(prev);
        }
    }

    // TODO: none of these actually use index; we'll need to
    // fix that to handle rewinding

    fn take_current(&mut self) -> Completion {
        self.index -= 1;
        self.history.pop().expect("No history to take from")
    }

    fn advance(&mut self) -> Option<Completion> {
        if let Some(completions) = &mut self.completions {
            if let Some(next) = completions.next() {
                return Some(next);
            }
        }

        self.completions = None;

        None
    }

    fn push_history(&mut self, item: Completion) {
        self.history.push(item);
        self.index = self.history.len()
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
}
