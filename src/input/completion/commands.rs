use super::{CompletableContext, Completer, Completion, CompletionContext};
use genawaiter::{sync::gen, yield_};

pub struct CommandsCompleter;

impl Completer for CommandsCompleter {
    type Iter = Box<dyn Iterator<Item = Completion>>;

    fn suggest(&self, context: CompletionContext) -> Self::Iter {
        let input = context.word().to_string();
        Box::new(
            gen!({
                yield_!(context.create_completion("quit".to_string()));
            })
            .into_iter()
            .filter(move |candidate| candidate.replacement.starts_with(&input)),
        )
    }
}
