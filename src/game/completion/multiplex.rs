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

pub struct MultiplexCompletionSource<T: Completer> {
    pub sources: Vec<T>,
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

impl<T: Completer> Completer for MultiplexCompletionSource<T> {
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

impl<T: CompletionSource> CompletionSource for MultiplexCompletionSource<T> {
    fn process(&mut self, text: String) {
        for source in &mut self.sources {
            source.process(text.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use crate::input::completion::tests::StaticCompleter;

    #[derive(Clone)]
    pub struct TestMultiplexSelector {
        indices: Vec<usize>,
    }

    impl TestMultiplexSelector {
        pub fn new(indices: Vec<usize>) -> Self {
            Self { indices }
        }
    }

    impl MultiplexSelectorFactory for TestMultiplexSelector {
        fn create(
            &self,
            _: crate::input::completion::CompletionContext,
        ) -> Box<(dyn MultiplexSelector + 'static)> {
            Box::new(self.clone())
        }
    }

    impl MultiplexSelector for TestMultiplexSelector {
        fn select(&mut self, _: Vec<Option<&Completion>>) -> usize {
            if self.indices.is_empty() {
                0
            } else {
                self.indices.remove(0)
            }
        }
    }

    #[test]
    fn multiplex_navigation() {
        let sources: Vec<Box<dyn Completer>> = vec![
            Box::new(StaticCompleter::new(vec![
                "alpastor".to_string(),
                "chorizo".to_string(),
            ])),
            Box::new(StaticCompleter::new(vec![
                "burrito".to_string(),
                "taco".to_string(),
            ])),
        ];
        let selector_factory = TestMultiplexSelector::new(vec![0, 1, 1, 0]);
        let multiplex = MultiplexCompletionSource {
            sources,
            selector_factory: Box::new(selector_factory),
        };
        let mut app = window("");
        let context = CompletionContext::from(&mut app);
        let completions: Vec<String> = multiplex
            .suggest(Box::new(&app), context)
            .map(|c| c.replacement)
            .collect();
        assert_eq!(completions, vec!["alpastor", "burrito", "taco", "chorizo"]);
    }
}
