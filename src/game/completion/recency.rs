use ritelinked::LinkedHashSet;

use crate::game::completion::flagged::SimpleCompletionSource;

use super::tokens::CompletionTokenizable;

const DEFAULT_MAX_ENTRIES: usize = 5000;

pub struct RecencyCompletionSource {
    max_entries: usize,
    entries: LinkedHashSet<String>,
}

impl Default for RecencyCompletionSource {
    fn default() -> Self {
        Self::with_max_entries(DEFAULT_MAX_ENTRIES)
    }
}

impl RecencyCompletionSource {
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: LinkedHashSet::default(),
        }
    }
}

crate::impl_simple_completer!(
    RecencyCompletionSource (&self, _app, _context) {
        // NOTE: It seems like there should be a way to do this without
        // unsafe, but I'm at a loss. This *should* be safe, because the
        // completion suggestions shouldn't be able to outlive the Buffer,
        // and the CompletionSource *should* live as long as the Buffer...
        let entries = &self.entries as *const LinkedHashSet<String>;
        unsafe {
            (*entries).iter()
                .rev()
                .map(|entry| entry.to_string())
        }
    }
);

impl SimpleCompletionSource for RecencyCompletionSource {
    fn process(&mut self, text: String) {
        for word in text.to_completion_tokens() {
            self.entries.insert(word.to_string());
        }
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use crate::input::completion::{BoxedSuggestions, Completer, CompletionContext};

    pub fn suggest_in_window<T: Completer>(
        source: &mut T,
        window_content: &'static str,
    ) -> BoxedSuggestions {
        let mut app = window(window_content);
        let context = CompletionContext::from(&mut app);
        return source.suggest(Box::new(&app), context);
    }

    pub fn suggest<T: Completer>(source: &mut T) -> BoxedSuggestions {
        suggest_in_window(source, "")
    }

    #[test]
    pub fn suggest_by_recency() {
        let mut source = RecencyCompletionSource::default();
        source.process("alpastor taco and chorizo burrito".to_string());

        let mut suggestions = suggest(&mut source);
        assert_eq!(suggestions.next().unwrap().replacement, "burrito");
        assert_eq!(suggestions.next().unwrap().replacement, "chorizo");
    }

    #[test]
    pub fn limit_to_most_recent() {
        let mut source = RecencyCompletionSource::with_max_entries(2);
        source.process("alpastor taco and chorizo burrito".to_string());

        let mut suggestions = suggest(&mut source);
        assert_eq!(suggestions.next().unwrap().replacement, "burrito");
        assert_eq!(suggestions.next().unwrap().replacement, "chorizo");
    }
}
