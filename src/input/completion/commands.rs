use crate::declare_simple_completer;
use genawaiter::{sync::gen, yield_};

use super::empty::EmptyCompleter;
use super::Completer;

declare_simple_completer!(
    CommandNamesCompleter (app, context) {
        let names: Vec<String> = app.commands()
            .names()
            .map(|v| v.to_string())
            .collect();

        gen!({
            // NOTE: this generator is obviously not doing much work
            // here, but more complicated completers might benefit...
            for name in names {
                yield_!(name);
            }
        })
    }
);

pub struct CommandsCompleter;

impl Completer for CommandsCompleter {
    fn suggest(
        &self,
        app: Box<&dyn super::CompletableContext>,
        context: super::CompletionContext,
    ) -> super::BoxedSuggestions {
        let (start, _) = context.word_range();
        if start <= 1 {
            return CommandNamesCompleter.suggest(app, context);
        }

        // NOTE: this 1 skips the `:`
        let command = context.nth_word(0).unwrap()[1..].to_string();
        if let Some(spec) = app.commands().get(&command) {
            if let Some(completer) = spec.completer.as_ref() {
                return completer.suggest(app, context);
            }
        }

        // fallback if no completions are available
        EmptyCompleter.suggest(app, context)
    }
}
