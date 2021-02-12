use super::{CompletableContext, Completer, Completion};

pub struct CommandsCompleter;

impl<T: CompletableContext> Completer<T> for CommandsCompleter {
    type Iter = Iter;

    fn suggest(&self, context: &T) -> Self::Iter {
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
