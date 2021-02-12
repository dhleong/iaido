use super::{CompletableContext, Completer, Completion, CompletionContext};

pub struct CommandsCompleter;

impl<T: CompletableContext> Completer<T> for CommandsCompleter {
    type Iter = Iter;
    // type Iter = Box<dyn Iterator<Item = Completion>>;

    fn suggest(&self, context: &CompletionContext<T>) -> Self::Iter {
        // Iter {
        //     names: context.commands().names().map(|n| n.clone()),
        // }

        // Box::new(
        //     context
        //         .context
        //         .commands()
        //         .names()
        //         .map(|n| context.create_completion(n.clone())),
        // )

        Iter {}
    }
}

pub struct Iter {}

impl Iterator for Iter {
    type Item = Completion;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
