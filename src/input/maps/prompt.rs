use std::rc::Rc;

use crate::input::{commands::CommandHandler, completion::Completer};

pub struct PromptConfig {
    pub prompt: String,
    pub history_key: String,
    pub handler: Box<CommandHandler>,
    pub completer: Option<Rc<dyn Completer>>,
}
