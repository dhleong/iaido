pub mod core;

use std::time::Duration;

use self::core::quit;

use super::{maps::KeyResult, Key, KeyError, KeySource, KeymapContext};

pub type CommandHandler = dyn Fn(CommandHandlerContext<'_>) -> KeyResult;

pub struct CommandHandlerContext<'a> {
    context: Box<&'a mut dyn KeymapContext>,
    input: String,
}

impl<'a> CommandHandlerContext<'a> {
    pub fn new<T: KeymapContext>(context: &'a mut T, input: String) -> Self {
        Self {
            context: Box::new(context),
            input,
        }
    }
}

impl KeymapContext for CommandHandlerContext<'_> {
    fn state(&self) -> &crate::app::State {
        self.context.state()
    }
    fn state_mut(&mut self) -> &mut crate::app::State {
        self.context.state_mut()
    }
}

impl KeySource for CommandHandlerContext<'_> {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
        self.context.poll_key(timeout)
    }
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.context.next_key()
    }
}

pub fn handle_command(context: CommandHandlerContext) -> KeyResult {
    let input_text = context.input.clone();

    match input_text.as_ref() {
        // TODO better dispatch
        "q" | "quit" => quit(context),

        _ => Err(KeyError::NoSuchCommand(input_text)),
    }
}
