use genawaiter::{sync::gen, yield_};

pub mod weighted;

use crate::game::completion::CompletionSource;
use crate::input::completion::empty::EmptyCompleter;
use crate::input::completion::{
    BoxedSuggestions, CompletableContext, Completer, Completion, CompletionContext,
};
use std::iter::Peekable;

pub trait MultiplexSelector {
    /// At least one item in [candidates] is guaranteed by the caller to be
    /// non-None; the index returned MUST point to an entry in candidates that
    /// is also non-None.
    fn select(&mut self, candidates: Vec<Option<&Completion>>) -> usize;
}

pub trait MultiplexSelectorFactory {
    fn create(&self, context: CompletionContext) -> Box<dyn MultiplexSelector>;
}

pub struct MultiplexCompletionSource {
    pub sources: Vec<Box<dyn CompletionSource>>,
    pub selector_factory: Box<dyn MultiplexSelectorFactory>,
}

fn produce_next(
    sources: &mut Vec<Peekable<BoxedSuggestions>>,
    selector: &mut Box<dyn MultiplexSelector>,
) -> Option<Completion> {
    let candidates: Vec<Option<&Completion>> =
        sources.iter_mut().map(|source| source.peek()).collect();
    if candidates.iter().all(|candidate| candidate.is_none()) {
        // No more candidates at all!
        return None;
    }

    let selected_index = selector.select(candidates);
    return Some(
        sources[selected_index]
            .next()
            .expect("Selected index was None!"),
    );
}

impl Completer for MultiplexCompletionSource {
    fn suggest(
        &self,
        app: Box<&dyn CompletableContext>,
        context: CompletionContext,
    ) -> BoxedSuggestions {
        let mut selector = self.selector_factory.create(context.clone());

        let mut sources: Vec<Peekable<BoxedSuggestions>> = self
            .sources
            .iter()
            .map(|source| source.suggest(app.clone(), context.clone()).peekable())
            .collect();
        if sources.is_empty() {
            EmptyCompleter.suggest(app, context.clone())
        } else {
            Box::new(
                gen!({
                    loop {
                        if let Some(suggestion) = produce_next(&mut sources, &mut selector) {
                            yield_!(suggestion);
                        } else {
                            // No more suggestions
                            break;
                        }
                    }
                })
                .into_iter(),
            )
        }
    }
}

impl CompletionSource for MultiplexCompletionSource {
    fn process(&mut self, text: String) {
        for source in &mut self.sources {
            source.process(text.to_string());
        }
    }
}
