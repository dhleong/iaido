use crate::editing::motion::MotionContext;

use super::{Completer, Completion};

pub struct CommandsCompleter;

impl<T: MotionContext> Completer<T> for CommandsCompleter {
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
