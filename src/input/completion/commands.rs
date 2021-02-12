use super::{CompletableContext, Completer, Completion, CompletionContext};
use genawaiter::{sync::gen, yield_};

pub struct CommandsCompleter;

impl Completer for CommandsCompleter {
    type Iter = Box<dyn Iterator<Item = Completion>>;

    fn suggest<T: CompletableContext>(&self, app: &T, context: CompletionContext) -> Self::Iter {
        let input = context.word().to_string();
        let names: Vec<String> = app.commands().names().map(|v| v.to_string()).collect();
        Box::new(
            gen!({
                for name in names {
                    yield_!(context.create_completion(name));
                }
            })
            .into_iter()
            .filter(move |candidate| candidate.replacement.starts_with(&input)),
        )
    }
}
